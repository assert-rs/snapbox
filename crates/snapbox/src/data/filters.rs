use crate::data::DataFormat;

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub(crate) struct FilterSet {
    flags: usize,
    against: Option<DataFormat>,
}

impl FilterSet {
    pub(crate) fn new() -> Self {
        Self::empty().redactions().newlines().paths()
    }

    pub(crate) const fn empty() -> Self {
        Self {
            flags: 0,
            against: None,
        }
    }

    pub(crate) fn redactions(mut self) -> Self {
        self.set(Self::REDACTIONS);
        self
    }

    pub(crate) fn newlines(mut self) -> Self {
        self.set(Self::NEWLINES);
        self
    }

    pub(crate) fn paths(mut self) -> Self {
        self.set(Self::PATHS);
        self
    }

    pub(crate) fn unordered(mut self) -> Self {
        self.set(Self::UNORDERED);
        self
    }

    pub(crate) fn against(mut self, format: DataFormat) -> Self {
        self.against = Some(format);
        self
    }

    pub(crate) const fn is_redaction_set(&self) -> bool {
        self.is_set(Self::REDACTIONS)
    }

    pub(crate) const fn is_newlines_set(&self) -> bool {
        self.is_set(Self::NEWLINES)
    }

    pub(crate) const fn is_paths_set(&self) -> bool {
        self.is_set(Self::PATHS)
    }

    pub(crate) const fn is_unordered_set(&self) -> bool {
        self.is_set(Self::UNORDERED)
    }

    pub(crate) const fn get_against(&self) -> Option<DataFormat> {
        self.against
    }
}

impl FilterSet {
    const REDACTIONS: usize = 1 << 0;
    const NEWLINES: usize = 1 << 1;
    const PATHS: usize = 1 << 2;
    const UNORDERED: usize = 1 << 3;

    fn set(&mut self, flag: usize) -> &mut Self {
        self.flags |= flag;
        self
    }

    const fn is_set(&self, flag: usize) -> bool {
        self.flags & flag != 0
    }
}
