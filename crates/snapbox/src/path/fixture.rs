/// Working directory for tests
#[derive(Debug)]
pub struct PathFixture(PathFixtureInner);

#[derive(Debug)]
enum PathFixtureInner {
    None,
    Immutable(std::path::PathBuf),
    #[cfg(feature = "path")]
    MutablePath(std::path::PathBuf),
    #[cfg(feature = "path")]
    MutableTemp {
        temp: tempfile::TempDir,
        path: std::path::PathBuf,
    },
}

impl PathFixture {
    pub fn none() -> Self {
        Self(PathFixtureInner::None)
    }

    pub fn immutable(target: &std::path::Path) -> Self {
        Self(PathFixtureInner::Immutable(target.to_owned()))
    }

    #[cfg(feature = "path")]
    pub fn mutable_temp() -> Result<Self, crate::assert::Error> {
        let temp = tempfile::tempdir().map_err(|e| e.to_string())?;
        // We need to get the `/private` prefix on Mac so variable substitutions work
        // correctly
        let path = crate::path::canonicalize(temp.path())
            .map_err(|e| format!("Failed to canonicalize {}: {}", temp.path().display(), e))?;
        Ok(Self(PathFixtureInner::MutableTemp { temp, path }))
    }

    #[cfg(feature = "path")]
    pub fn mutable_at(target: &std::path::Path) -> Result<Self, crate::assert::Error> {
        let _ = std::fs::remove_dir_all(target);
        std::fs::create_dir_all(target)
            .map_err(|e| format!("Failed to create {}: {}", target.display(), e))?;
        Ok(Self(PathFixtureInner::MutablePath(target.to_owned())))
    }

    #[cfg(feature = "path")]
    pub fn with_template(
        self,
        template_root: &std::path::Path,
    ) -> Result<Self, crate::assert::Error> {
        match &self.0 {
            PathFixtureInner::None | PathFixtureInner::Immutable(_) => {
                return Err("Sandboxing is disabled".into());
            }
            PathFixtureInner::MutablePath(path) | PathFixtureInner::MutableTemp { path, .. } => {
                crate::debug!(
                    "Initializing {} from {}",
                    path.display(),
                    template_root.display()
                );
                super::copy_template(template_root, path)?;
            }
        }

        Ok(self)
    }

    pub fn is_mutable(&self) -> bool {
        match &self.0 {
            PathFixtureInner::None | PathFixtureInner::Immutable(_) => false,
            #[cfg(feature = "path")]
            PathFixtureInner::MutablePath(_) => true,
            #[cfg(feature = "path")]
            PathFixtureInner::MutableTemp { .. } => true,
        }
    }

    pub fn path(&self) -> Option<&std::path::Path> {
        match &self.0 {
            PathFixtureInner::None => None,
            PathFixtureInner::Immutable(path) => Some(path.as_path()),
            #[cfg(feature = "path")]
            PathFixtureInner::MutablePath(path) => Some(path.as_path()),
            #[cfg(feature = "path")]
            PathFixtureInner::MutableTemp { path, .. } => Some(path.as_path()),
        }
    }

    /// Explicitly close to report errors
    pub fn close(self) -> Result<(), std::io::Error> {
        match self.0 {
            PathFixtureInner::None | PathFixtureInner::Immutable(_) => Ok(()),
            #[cfg(feature = "path")]
            PathFixtureInner::MutablePath(_) => Ok(()),
            #[cfg(feature = "path")]
            PathFixtureInner::MutableTemp { temp, .. } => temp.close(),
        }
    }
}

impl Default for PathFixture {
    fn default() -> Self {
        Self::none()
    }
}
