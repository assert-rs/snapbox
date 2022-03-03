/// Feature-flag controlled additional test debug information
#[cfg(feature = "debug")]
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => ({
        eprint!("[{:>w$}] \t", module_path!(), w = 28);
        eprintln!($($arg)*);
    })
}

/// Feature-flag controlled additional test debug information
#[cfg(not(feature = "debug"))]
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {};
}
