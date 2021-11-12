#[derive(Clone, Debug)]
pub(crate) struct BinRegistry {
    bins: std::collections::HashMap<String, crate::schema::Bin>,
    fallback: bool,
}

impl BinRegistry {
    pub(crate) fn new() -> Self {
        Self {
            bins: Default::default(),
            fallback: true,
        }
    }

    pub(crate) fn register_bin(&mut self, name: String, bin: crate::schema::Bin) {
        self.bins.insert(name, bin);
    }

    pub(crate) fn register_bins(
        &mut self,
        bins: impl Iterator<Item = (String, crate::schema::Bin)>,
    ) {
        self.bins.extend(bins);
    }

    pub(crate) fn resolve_bin(
        &self,
        bin: crate::schema::Bin,
    ) -> Result<crate::schema::Bin, crate::Error> {
        match bin {
            crate::schema::Bin::Path(path) => {
                let bin = crate::schema::Bin::Path(path);
                Ok(bin)
            }
            crate::schema::Bin::Name(name) => {
                let bin = self
                    .resolve_name(&name)
                    .ok_or_else(|| format!("Unknown bin.name = {}", name))?;
                Ok(bin)
            }
            crate::schema::Bin::Error(err) => Err(err),
        }
    }

    pub(crate) fn resolve_name(&self, name: &str) -> Option<crate::schema::Bin> {
        if let Some(path) = self.bins.get(name) {
            return Some(path.clone());
        }

        if self.fallback {
            return Some(crate::schema::Bin::Path(crate::cargo::cargo_bin(name)));
        }

        None
    }
}

impl Default for BinRegistry {
    fn default() -> Self {
        Self::new()
    }
}
