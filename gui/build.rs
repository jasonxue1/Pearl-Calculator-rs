fn main() {
    println!("cargo:rerun-if-changed=../assets/icon.ico");

    if std::env::var_os("CARGO_CFG_TARGET_OS").as_deref() == Some(std::ffi::OsStr::new("windows")) {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("../assets/icon.ico");
        res.set("ProductName", "Pearl Calculator");
        res.set("FileDescription", "A simple Pearl Calculator");
        res.set("Comments", "A simple Pearl Calculato built with egui");
        if let Err(err) = res.compile() {
            panic!("failed to compile Windows resources: {err}");
        }
    }
}
