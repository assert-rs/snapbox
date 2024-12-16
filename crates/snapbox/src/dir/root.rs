/// Working directory for tests
#[derive(Debug)]
pub struct DirRoot(DirRootInner);

#[derive(Debug)]
enum DirRootInner {
    None,
    Immutable(std::path::PathBuf),
    #[cfg(feature = "dir")]
    MutablePath(std::path::PathBuf),
    #[cfg(feature = "dir")]
    MutableTemp {
        temp: tempfile::TempDir,
        path: std::path::PathBuf,
    },
}

impl DirRoot {
    pub fn none() -> Self {
        Self(DirRootInner::None)
    }

    pub fn immutable(target: &std::path::Path) -> Self {
        Self(DirRootInner::Immutable(target.to_owned()))
    }

    #[cfg(feature = "dir")]
    pub fn mutable_temp() -> Result<Self, crate::assert::Error> {
        let temp = tempfile::tempdir().map_err(|e| e.to_string())?;
        // We need to get the `/private` prefix on Mac so variable substitutions work
        // correctly
        let path = crate::dir::canonicalize(temp.path())
            .map_err(|e| format!("Failed to canonicalize {}: {}", temp.path().display(), e))?;
        Ok(Self(DirRootInner::MutableTemp { temp, path }))
    }

    #[cfg(feature = "dir")]
    pub fn mutable_at(target: &std::path::Path) -> Result<Self, crate::assert::Error> {
        let _ = std::fs::remove_dir_all(target);
        std::fs::create_dir_all(target)
            .map_err(|e| format!("Failed to create {}: {}", target.display(), e))?;
        Ok(Self(DirRootInner::MutablePath(target.to_owned())))
    }

    #[cfg(feature = "dir")]
    pub fn with_template<F>(self, template: &F) -> Result<Self, crate::assert::Error>
    where
        F: crate::dir::DirFixture + ?Sized,
    {
        match &self.0 {
            DirRootInner::None | DirRootInner::Immutable(_) => {
                return Err("Sandboxing is disabled".into());
            }
            DirRootInner::MutablePath(path) | DirRootInner::MutableTemp { path, .. } => {
                crate::debug!("Initializing {} from {:?}", path.display(), template);
                template.write_to_path(path)?;
            }
        }

        Ok(self)
    }

    pub fn is_mutable(&self) -> bool {
        match &self.0 {
            DirRootInner::None | DirRootInner::Immutable(_) => false,
            #[cfg(feature = "dir")]
            DirRootInner::MutablePath(_) => true,
            #[cfg(feature = "dir")]
            DirRootInner::MutableTemp { .. } => true,
        }
    }

    pub fn path(&self) -> Option<&std::path::Path> {
        match &self.0 {
            DirRootInner::None => None,
            DirRootInner::Immutable(path) => Some(path.as_path()),
            #[cfg(feature = "dir")]
            DirRootInner::MutablePath(path) => Some(path.as_path()),
            #[cfg(feature = "dir")]
            DirRootInner::MutableTemp { path, .. } => Some(path.as_path()),
        }
    }

    /// Explicitly close to report errors
    pub fn close(self) -> Result<(), std::io::Error> {
        match self.0 {
            DirRootInner::None | DirRootInner::Immutable(_) => Ok(()),
            #[cfg(feature = "dir")]
            DirRootInner::MutablePath(_) => Ok(()),
            #[cfg(feature = "dir")]
            DirRootInner::MutableTemp { temp, .. } => temp.close(),
        }
    }
}

impl Default for DirRoot {
    fn default() -> Self {
        Self::none()
    }
}
