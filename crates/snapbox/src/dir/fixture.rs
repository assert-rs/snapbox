/// Collection of files
pub trait DirFixture {
    /// Initialize a test fixture directory `root`
    fn write_to_path(&self, root: &std::path::Path) -> Result<(), crate::assert::Error>;
}

#[cfg(feature = "dir")] // for documentation purposes only
impl DirFixture for std::path::Path {
    fn write_to_path(&self, root: &std::path::Path) -> Result<(), crate::assert::Error> {
        super::copy_template(self, root)
    }
}

#[cfg(feature = "dir")] // for documentation purposes only
impl DirFixture for &'_ std::path::Path {
    fn write_to_path(&self, root: &std::path::Path) -> Result<(), crate::assert::Error> {
        std::path::Path::new(self).write_to_path(root)
    }
}

#[cfg(feature = "dir")] // for documentation purposes only
impl DirFixture for &'_ std::path::PathBuf {
    fn write_to_path(&self, root: &std::path::Path) -> Result<(), crate::assert::Error> {
        std::path::Path::new(self).write_to_path(root)
    }
}

#[cfg(feature = "dir")] // for documentation purposes only
impl DirFixture for std::path::PathBuf {
    fn write_to_path(&self, root: &std::path::Path) -> Result<(), crate::assert::Error> {
        std::path::Path::new(self).write_to_path(root)
    }
}

#[cfg(feature = "dir")] // for documentation purposes only
impl DirFixture for str {
    fn write_to_path(&self, root: &std::path::Path) -> Result<(), crate::assert::Error> {
        std::path::Path::new(self).write_to_path(root)
    }
}

#[cfg(feature = "dir")] // for documentation purposes only
impl DirFixture for &'_ str {
    fn write_to_path(&self, root: &std::path::Path) -> Result<(), crate::assert::Error> {
        std::path::Path::new(self).write_to_path(root)
    }
}

#[cfg(feature = "dir")] // for documentation purposes only
impl DirFixture for &'_ String {
    fn write_to_path(&self, root: &std::path::Path) -> Result<(), crate::assert::Error> {
        std::path::Path::new(self).write_to_path(root)
    }
}

#[cfg(feature = "dir")] // for documentation purposes only
impl DirFixture for String {
    fn write_to_path(&self, root: &std::path::Path) -> Result<(), crate::assert::Error> {
        std::path::Path::new(self).write_to_path(root)
    }
}
