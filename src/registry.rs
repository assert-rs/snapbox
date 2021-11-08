#[derive(Clone, Debug)]
pub(crate) struct BinRegistry {
    fallback: bool,
}

impl BinRegistry {
    pub(crate) fn new() -> Self {
        Self { fallback: true }
    }

    pub(crate) fn resolve_bin(&self, bin: crate::Bin) -> Result<crate::Bin, String> {
        match bin {
            crate::Bin::Path(path) => {
                let bin = crate::Bin::Path(path);
                Ok(bin)
            }
            crate::Bin::Name(name) => {
                let path = self
                    .resolve_name(&name)
                    .ok_or_else(|| format!("Unknown bin.name = {}", name))?;
                let bin = crate::Bin::Path(path);
                Ok(bin)
            }
        }
    }

    pub(crate) fn resolve_name(&self, name: &str) -> Option<std::path::PathBuf> {
        if self.fallback {
            return Some(crate::cargo_bin(name));
        }

        None
    }
}

impl Default for BinRegistry {
    fn default() -> Self {
        Self::new()
    }
}
