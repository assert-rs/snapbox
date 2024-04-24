mod lines;

pub use lines::LinesWithTerminator;

#[doc(inline)]
pub use crate::cargo_rustc_current_dir;
#[doc(inline)]
pub use crate::current_dir;
#[doc(inline)]
pub use crate::current_rs;

pub fn display_relpath(path: impl AsRef<std::path::Path>) -> String {
    let path = path.as_ref();
    let relpath = if let Ok(cwd) = std::env::current_dir() {
        match path.strip_prefix(cwd) {
            Ok(path) => path,
            Err(_) => path,
        }
    } else {
        path
    };
    relpath.display().to_string()
}
