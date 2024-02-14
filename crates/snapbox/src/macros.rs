/// Find the directory for your source file
#[doc(hidden)] // forced to be visible in intended location
#[macro_export]
macro_rules! current_dir {
    () => {{
        let root = if let Some(rustc_root) = ::std::option_env!("CARGO_RUSTC_CURRENT_DIR") {
            ::std::path::Path::new(rustc_root)
        } else {
            let manifest_dir = ::std::path::Path::new(::std::env!("CARGO_MANIFEST_DIR"));
            manifest_dir
                .ancestors()
                .filter(|it| it.join("Cargo.toml").exists())
                .last()
                .unwrap()
        };
        let file = ::std::file!();
        let rel_path = ::std::path::Path::new(file).parent().unwrap();
        root.join(rel_path)
    }};
}
