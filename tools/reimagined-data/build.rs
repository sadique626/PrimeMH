// Gives ReimaginedData.exe the d2r-reimagined.com icon (shown in Explorer and the taskbar).
fn main() {
    println!("cargo:rerun-if-changed=icon.ico");
    if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
        winres::WindowsResource::new().set_icon("icon.ico").compile().expect("could not embed icon");
    }
}
