use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "boot")]
#[command(about = "Kernel build/run orchestration")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Build,
    RunQemu,
    RunQemuDebug,
    RunHardwareArtifact,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Build => build(),
        Commands::RunQemu => run_qemu(false),
        Commands::RunQemuDebug => run_qemu(true),
        Commands::RunHardwareArtifact => run_hardware_artifact(),
    }
}

fn build() -> Result<()> {
    let root = boot::workspace_root();
    run_cargo(
        &root,
        &[
            "build",
            "--release",
            "-p",
            "kernel",
            "--target",
            "targets/x86_64-kernel.json",
        ],
    )?;
    run_cargo(
        &root,
        &[
            "build",
            "--release",
            "-p",
            "user-smoke",
            "--target",
            "targets/x86_64-user.json",
        ],
    )?;
    stage_artifacts(&root)?;
    println!("Build complete: {}", root.join("artifacts").display());
    Ok(())
}

fn run_qemu(debug: bool) -> Result<()> {
    let root = boot::workspace_root();
    build()?;

    let mut cmd = Command::new("qemu-system-x86_64");
    cmd.current_dir(&root)
        .arg("-machine")
        .arg("q35")
        .arg("-cpu")
        .arg("qemu64")
        .arg("-m")
        .arg("1024")
        .arg("-serial")
        .arg("stdio")
        .arg("-display")
        .arg("none")
        .arg("-drive")
        .arg("format=raw,file=fat:rw:artifacts/efi");

    if let Some(ovmf) = find_ovmf_path() {
        cmd.arg("-bios").arg(ovmf);
    }
    if debug {
        cmd.arg("-s").arg("-S");
    }

    let status = cmd.status().context("failed to start qemu")?;
    if !status.success() {
        bail!("qemu exited with status {status}");
    }
    Ok(())
}

fn run_hardware_artifact() -> Result<()> {
    let root = boot::workspace_root();
    build()?;

    let dist = root.join("dist");
    fs::create_dir_all(&dist)?;
    let zip_path = dist.join("kernel-uefi.zip");
    if zip_path.exists() {
        fs::remove_file(&zip_path)?;
    }

    let powershell =
        "Compress-Archive -Path artifacts/efi/* -DestinationPath dist/kernel-uefi.zip -Force";
    let status = Command::new("powershell")
        .current_dir(&root)
        .args(["-NoProfile", "-Command", powershell])
        .status()
        .context("failed to create hardware zip with powershell Compress-Archive")?;
    if !status.success() {
        bail!("artifact archive creation failed: {status}");
    }

    println!("Hardware artifact ready: {}", zip_path.display());
    Ok(())
}

fn stage_artifacts(root: &Path) -> Result<()> {
    let artifacts = root.join("artifacts");
    let efi_dir = artifacts.join("efi");
    let efi_boot_dir = efi_dir.join("EFI").join("BOOT");

    fs::create_dir_all(&efi_boot_dir)?;
    write_limine_config(&efi_dir)?;

    let kernel_src = find_artifact(
        &root.join("target").join("x86_64-kernel").join("release"),
        "kernel",
    )
    .ok_or_else(|| anyhow::anyhow!("kernel artifact not found; run `cargo boot-build` first"))?;
    let user_src = find_artifact(
        &root.join("target").join("x86_64-user").join("release"),
        "user-smoke",
    )
    .ok_or_else(|| {
        anyhow::anyhow!("user-smoke artifact not found; run `cargo boot-build` first")
    })?;

    fs::copy(&kernel_src, efi_dir.join("kernel.elf")).with_context(|| {
        format!(
            "failed to copy kernel artifact from {}",
            kernel_src.to_string_lossy()
        )
    })?;
    fs::copy(&user_src, efi_dir.join("user-smoke.elf")).with_context(|| {
        format!(
            "failed to copy user smoke artifact from {}",
            user_src.to_string_lossy()
        )
    })?;

    let limine_bootx64 = ensure_limine_bootx64(root)?;
    fs::copy(limine_bootx64, efi_boot_dir.join("BOOTX64.EFI"))?;
    Ok(())
}

fn ensure_limine_bootx64(root: &Path) -> Result<PathBuf> {
    let limine_dir = root.join("limine");
    if !limine_dir.exists() {
        let status = Command::new("git")
            .current_dir(root)
            .args([
                "clone",
                "--depth",
                "1",
                "--branch",
                "v8.x-binary",
                "https://github.com/limine-bootloader/limine.git",
                "limine",
            ])
            .status()
            .context("failed to clone limine binary repo")?;
        if !status.success() {
            bail!("limine clone failed with status {status}");
        }
    }

    find_file_named(&limine_dir, "BOOTX64.EFI").ok_or_else(|| {
        anyhow::anyhow!(
            "could not find BOOTX64.EFI in limine checkout ({})",
            limine_dir.display()
        )
    })
}

fn write_limine_config(efi_dir: &Path) -> Result<()> {
    let content = r#"timeout: 0
default_entry: 0

/Kernel Research v1
    protocol: limine
    kernel_path: boot():/kernel.elf
    module_path: boot():/user-smoke.elf
    kernel_cmdline: log=serial
"#;

    fs::write(efi_dir.join("limine.conf"), content)?;
    Ok(())
}

fn find_ovmf_path() -> Option<String> {
    if let Ok(path) = std::env::var("OVMF_PATH") {
        if !path.is_empty() {
            return Some(path);
        }
    }

    let candidates = [
        r"C:\Program Files\qemu\share\edk2-x86_64-code.fd",
        r"C:\Program Files\qemu\share\OVMF.fd",
    ];
    for candidate in candidates {
        if Path::new(candidate).exists() {
            return Some(candidate.to_string());
        }
    }
    None
}

fn find_file_named(dir: &Path, filename: &str) -> Option<PathBuf> {
    let mut stack = vec![dir.to_path_buf()];
    while let Some(path) = stack.pop() {
        let entries = fs::read_dir(&path).ok()?;
        for entry in entries {
            let entry = entry.ok()?;
            let p = entry.path();
            if p.is_dir() {
                stack.push(p);
                continue;
            }
            if p.file_name() == Some(OsStr::new(filename)) {
                return Some(p);
            }
        }
    }
    None
}

fn run_cargo(root: &Path, args: &[&str]) -> Result<()> {
    let status = Command::new("cargo")
        .current_dir(root)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("failed to launch cargo subprocess")?;
    if !status.success() {
        bail!("cargo {:?} failed with status {status}", args);
    }
    Ok(())
}

fn find_artifact(dir: &Path, stem: &str) -> Option<PathBuf> {
    let bare = dir.join(stem);
    if bare.exists() {
        return Some(bare);
    }
    let exe = dir.join(format!("{stem}.exe"));
    if exe.exists() {
        return Some(exe);
    }
    None
}
