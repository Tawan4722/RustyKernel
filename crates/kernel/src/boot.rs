use limine::BaseRevision;
use limine::file::File;
use limine::memory_map::Entry;
use limine::request::{
    FramebufferRequest, HhdmRequest, MemoryMapRequest, ModuleRequest, RequestsEndMarker,
    RequestsStartMarker, StackSizeRequest,
};

pub struct BootSnapshot {
    pub hhdm_offset: u64,
    pub memory_map: &'static [&'static Entry],
    pub modules: &'static [&'static File],
}

#[used]
#[link_section = ".limine_reqs"]
static BASE_REVISION: BaseRevision = BaseRevision::new();

#[used]
#[link_section = ".limine_req_start"]
static REQ_START: RequestsStartMarker = RequestsStartMarker::new();

#[used]
#[link_section = ".limine_reqs"]
static STACK_SIZE_REQUEST: StackSizeRequest = StackSizeRequest::new().with_size(1024 * 1024);

#[used]
#[link_section = ".limine_reqs"]
static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

#[used]
#[link_section = ".limine_reqs"]
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

#[used]
#[link_section = ".limine_reqs"]
static MEMORY_MAP_REQUEST: MemoryMapRequest = MemoryMapRequest::new();

#[used]
#[link_section = ".limine_reqs"]
static MODULE_REQUEST: ModuleRequest = ModuleRequest::new();

#[used]
#[link_section = ".limine_req_end"]
static REQ_END: RequestsEndMarker = RequestsEndMarker::new();

pub fn snapshot() -> Result<BootSnapshot, &'static str> {
    let hhdm_resp = HHDM_REQUEST
        .get_response()
        .ok_or("no HHDM response from limine")?;
    let memmap_resp = MEMORY_MAP_REQUEST
        .get_response()
        .ok_or("no memory map response from limine")?;
    let module_resp = MODULE_REQUEST
        .get_response()
        .ok_or("no module response from limine")?;

    if !BASE_REVISION.is_supported() && !BASE_REVISION.is_valid() {
        return Err("unsupported limine base revision");
    }

    let modules = module_resp.modules();
    let _ = FRAMEBUFFER_REQUEST.get_response();

    Ok(BootSnapshot {
        hhdm_offset: hhdm_resp.offset(),
        memory_map: memmap_resp.entries(),
        modules,
    })
}
