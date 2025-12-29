//! Initialization that needs to be done on startup
/// Performs any global state initialization that needs to be done before performing image operations
pub fn init() {
    let jxl = jxl_oxide::integration::register_decoding_hook();
    let dds = image_dds::register_decoding_hook();
    log::warn!("jxl is {}", jxl);
    log::warn!("dds is {}", dds);
}
