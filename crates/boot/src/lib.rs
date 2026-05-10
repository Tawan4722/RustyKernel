pub fn workspace_root() -> std::path::PathBuf {
    std::env::current_dir().expect("current_dir unavailable")
}
