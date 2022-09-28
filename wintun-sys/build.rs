use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=wintun/wintun_functions.h");

    #[cfg(target_os = "windows")]
    {
        let bindings = bindgen::Builder::default()
            .header("wintun/wintun_functions.h")
            .parse_callbacks(Box::new(bindgen::CargoCallbacks))
            .allowlist_function("Wintun.*")
            .allowlist_type("WINTUN_.*")
            .allowlist_var("WINTUN_.*")
            .blocklist_type("_GUID")
            .blocklist_type("BOOL")
            .blocklist_type("BYTE")
            .blocklist_type("DWORD")
            .blocklist_type("DWORD64")
            .blocklist_type("GUID")
            .blocklist_type("HANDLE")
            .blocklist_type("LPCWSTR")
            .blocklist_type("NET_LUID")
            .blocklist_type("WCHAR")
            .blocklist_type("wchar_t")
            .dynamic_library_name("wintun")
            .dynamic_link_require_all(true)
            .opaque_type("NET_LUID")
            .generate()
            .expect("Unable to generate bindings");

        // Write the bindings to the $OUT_DIR/bindings.rs file.
        let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
        bindings
            .write_to_file(out_path.join("bindings.rs"))
            .expect("Couldn't write bindings!");
    }
}
