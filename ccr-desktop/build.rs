//! Embed the CCR icon as Windows exe resource ID 1 so the taskbar, Alt-Tab and
//! the GPUI window class (which loads MAKEINTRESOURCE(1)) show the brand mark.
//! Other targets keep their existing behavior; no runtime icon API is claimed.
fn main() {
    println!("cargo:rerun-if-changed=assets/ccr.rc");
    println!("cargo:rerun-if-changed=assets/ccr.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
        && let Err(error) =
            embed_resource::compile("assets/ccr.rc", embed_resource::NONE).manifest_required()
    {
        panic!("CCR Windows icon resource compilation failed: {error:?}");
    }
}
