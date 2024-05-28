/// Check if a value is the same as an expected value
///
/// By default [`filters`][crate::filter] are applied, including:
/// - `...` is a line-wildcard when on a line by itself
/// - `[..]` is a character-wildcard when inside a line
/// - `[EXE]` matches `.exe` on Windows
/// - `"{...}"` is a JSON value wildcard
/// - `"...": "{...}"` is a JSON key-value wildcard
/// - `\` to `/`
/// - Newlines
///
/// To limit this to newline normalization for text, call [`Data::raw`][crate::Data] on `expected`.
///
/// # Effective signature
///
/// ```rust
/// # use snapbox::IntoData;
/// fn assert_data_eq(actual: impl IntoData, expected: impl IntoData) {
///     // ...
/// }
/// ```
///
/// # Examples
///
/// ```rust
/// # use snapbox::assert_data_eq;
/// let output = "something";
/// let expected = "so[..]g";
/// assert_data_eq!(output, expected);
/// ```
///
/// Can combine this with [`file!`]
/// ```rust,no_run
/// # use snapbox::assert_data_eq;
/// # use snapbox::file;
/// let actual = "something";
/// assert_data_eq!(actual, file!["output.txt"]);
/// ```
#[macro_export]
macro_rules! assert_data_eq {
    ($actual: expr, $expected: expr $(,)?) => {{
        let actual = $crate::IntoData::into_data($actual);
        let expected = $crate::IntoData::into_data($expected);
        $crate::Assert::new()
            .action_env($crate::assert::DEFAULT_ACTION_ENV)
            .eq(actual, expected);
    }};
}

/// Find the directory for your source file
#[doc(hidden)] // forced to be visible in intended location
#[macro_export]
macro_rules! current_dir {
    () => {{
        let root = $crate::utils::cargo_rustc_current_dir!();
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
        let root = $crate::utils::cargo_rustc_current_dir!();
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
        })();
    }

    #[test]
    fn nested_fn_path() {
        fn nested() {
            assert_eq!(fn_path!(), "snapbox::macros::test::nested_fn_path::nested");
        }
        nested();
    }
}
