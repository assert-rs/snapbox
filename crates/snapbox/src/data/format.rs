/// Describes the structure of [`Data`][crate::Data]
#[derive(Clone, Debug, PartialEq, Eq, Copy, Hash, Default)]
#[non_exhaustive]
pub enum DataFormat {
    /// Processing of the [`Data`][crate::Data] failed
    Error,
    /// Non-textual, opaque data
    Binary,
    #[default]
    Text,
    #[cfg(feature = "json")]
    Json,
    /// Streamed JSON output according to <https://jsonlines.org/>
    #[cfg(feature = "json")]
    JsonLines,
    /// [ANSI escape codes](https://en.wikipedia.org/wiki/ANSI_escape_code#DOS_and_Windows)
    /// rendered as [svg](https://docs.rs/anstyle-svg)
    #[cfg(feature = "term-svg")]
    TermSvg,
}

impl DataFormat {
    /// Assumed file extension for the format
    pub fn ext(self) -> &'static str {
        match self {
            Self::Error => "txt",
            Self::Binary => "bin",
            Self::Text => "txt",
            #[cfg(feature = "json")]
            Self::Json => "json",
            #[cfg(feature = "json")]
            Self::JsonLines => "jsonl",
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
            #[cfg(feature = "json")]
            "jsonl" => DataFormat::JsonLines,
            #[cfg(feature = "term-svg")]
            "term.svg" => Self::TermSvg,
            _ => DataFormat::Text,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn combos() {
        #[cfg(feature = "json")]
        let json = DataFormat::Json;
        #[cfg(not(feature = "json"))]
        let json = DataFormat::Text;
        #[cfg(feature = "json")]
        let jsonl = DataFormat::JsonLines;
        #[cfg(not(feature = "json"))]
        let jsonl = DataFormat::Text;
        #[cfg(feature = "term-svg")]
        let term_svg = DataFormat::TermSvg;
        #[cfg(not(feature = "term-svg"))]
        let term_svg = DataFormat::Text;
        let cases = [
            ("foo", DataFormat::Text),
            (".foo", DataFormat::Text),
            ("foo.txt", DataFormat::Text),
            (".foo.txt", DataFormat::Text),
            ("foo.stdout.txt", DataFormat::Text),
            ("foo.json", json),
            ("foo.stdout.json", DataFormat::Text),
            (".foo.json", DataFormat::Text),
            ("foo.jsonl", jsonl),
            ("foo.stdout.jsonl", DataFormat::Text),
            (".foo.jsonl", DataFormat::Text),
            ("foo.term.svg", term_svg),
            ("foo.stdout.term.svg", DataFormat::Text),
            (".foo.term.svg", DataFormat::Text),
        ];
        for (input, output) in cases {
            let input = std::path::Path::new(input);
            assert_eq!(DataFormat::from(input), output, "for `{}`", input.display());
        }
    }
}
