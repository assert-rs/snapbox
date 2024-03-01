/// Find the directory for your source file
#[doc(hidden)] // forced to be visible in intended location
#[macro_export]
macro_rules! current_dir {
    () => {{
        let root = $crate::path::cargo_rustc_current_dir!();
        let file = ::std::file!();
        let rel_path = ::std::path::Path::new(file).parent().unwrap();
        root.join(rel_path)
    }};
}

/// Find the directory for your source file
#[doc(hidden)] // forced to be visible in intended location
#[macro_export]
macro_rules! current_rs {
    () => {{
        let root = $crate::path::cargo_rustc_current_dir!();
        let file = ::std::file!();
        let rel_path = ::std::path::Path::new(file);
        root.join(rel_path)
    }};
}

/// Find the base directory for [`std::file!`]
#[doc(hidden)] // forced to be visible in intended location
#[macro_export]
macro_rules! cargo_rustc_current_dir {
    () => {{
        if let Some(rustc_root) = ::std::option_env!("CARGO_RUSTC_CURRENT_DIR") {
            ::std::path::Path::new(rustc_root)
        } else {
            let manifest_dir = ::std::path::Path::new(::std::env!("CARGO_MANIFEST_DIR"));
            manifest_dir
                .ancestors()
                .filter(|it| it.join("Cargo.toml").exists())
                .last()
                .unwrap()
        }
    }};
}

/// Path to the current function
///
/// Closures are ignored
#[doc(hidden)]
#[macro_export]
macro_rules! fn_path {
    () => {{
        fn f() {}
        fn type_name_of_val<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let mut name = type_name_of_val(f).strip_suffix("::f").unwrap_or("");
        while let Some(rest) = name.strip_suffix("::{{closure}}") {
            name = rest;
        }
        name
    }};
}

#[cfg(test)]
mod test {
    #[test]
    fn direct_fn_path() {
        assert_eq!(fn_path!(), "snapbox::macros::test::direct_fn_path");
    }

    #[test]
    #[allow(clippy::redundant_closure_call)]
    fn closure_fn_path() {
        (|| {
            assert_eq!(fn_path!(), "snapbox::macros::test::closure_fn_path");
        })()
    }

    #[test]
    fn nested_fn_path() {
        fn nested() {
            assert_eq!(fn_path!(), "snapbox::macros::test::nested_fn_path::nested");
        }
        nested()
    }
}
