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

impl From<&std::path::Path> for DataFormat {
    fn from(path: &std::path::Path) -> Self {
        let file_name = path
            .file_name()
            .and_then(|e| e.to_str())
            .unwrap_or_default();
        let (file_stem, mut ext) = file_name.split_once('.').unwrap_or((file_name, ""));
        if file_stem.is_empty() {
            (_, ext) = file_stem.split_once('.').unwrap_or((file_name, ""));
        }
        match ext {
            #[cfg(feature = "json")]
            "json" => DataFormat::Json,
            #[cfg(feature = "term-svg")]
            "term.svg" => Self::TermSvg,
            _ => DataFormat::Text,
        }
    }
}
