#[cfg(feature = "debug")]
#[allow(unused_macros)]
macro_rules! debug {
    ($($arg:tt)*) => ({
        eprint!("[{:>w$}] \t", module_path!(), w = 28);
        eprintln!($($arg)*);
    })
}

#[cfg(not(feature = "debug"))]
#[allow(unused_macros)]
macro_rules! debug {
    ($($arg:tt)*) => {};
}
