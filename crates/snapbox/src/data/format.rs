#[derive(Clone, Debug, PartialEq, Eq, Copy, Hash, Default)]
pub enum DataFormat {
    Error,
    Binary,
    #[default]
    Text,
    #[cfg(feature = "json")]
    Json,
    #[cfg(feature = "term-svg")]
    TermSvg,
}

impl DataFormat {
    pub fn ext(self) -> &'static str {
        match self {
            Self::Error => "txt",
            Self::Binary => "bin",
            Self::Text => "txt",
            #[cfg(feature = "json")]
            Self::Json => "json",
            #[cfg(feature = "term-svg")]
            Self::TermSvg => "term.svg",
        }
    }
}
