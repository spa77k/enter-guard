fn main() {
    println!("cargo:rerun-if-changed=../assets/EnterGuard.ico");
    println!("cargo:rerun-if-changed=app.manifest");
    if cfg!(windows) {
        winresource::WindowsResource::new()
            .set_icon("../assets/EnterGuard.ico")
            .set_manifest_file("app.manifest")
            .set("FileDescription", "EnterGuard — Discord用Enter変換")
            .compile()
            .expect("Windows resources");
    }
}
