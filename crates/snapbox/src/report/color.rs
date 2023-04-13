#[derive(Copy, Clone, Debug, Default)]
pub struct Palette {
    pub(crate) info: anstyle::Style,
    pub(crate) warn: anstyle::Style,
    pub(crate) error: anstyle::Style,
    pub(crate) hint: anstyle::Style,
    pub(crate) expected: anstyle::Style,
    pub(crate) actual: anstyle::Style,
}

impl Palette {
    pub fn color() -> Self {
        if cfg!(feature = "color") {
            Self {
                info: anstyle::AnsiColor::Green.on_default(),
                warn: anstyle::AnsiColor::Yellow.on_default(),
                error: anstyle::AnsiColor::Red.on_default(),
                hint: anstyle::Effects::DIMMED.into(),
                expected: anstyle::AnsiColor::Green.on_default() | anstyle::Effects::UNDERLINE,
                actual: anstyle::AnsiColor::Red.on_default() | anstyle::Effects::UNDERLINE,
            }
        } else {
            Self::plain()
        }
    }

    pub fn plain() -> Self {
        Self::default()
    }

    #[deprecated(since = "0.4.9", note = "Renamed to `Palette::color")]
    pub fn always() -> Self {
        Self::color()
    }

    #[deprecated(since = "0.4.9", note = "Renamed to `Palette::plain")]
    pub fn never() -> Self {
        Self::plain()
    }

    #[deprecated(
        since = "0.4.9",
        note = "Use `Palette::always`, `auto` behavior is now implicit"
    )]
    pub fn auto() -> Self {
        if is_colored() {
            Self::color()
        } else {
            Self::plain()
        }
    }

    pub fn info<D: std::fmt::Display>(self, item: D) -> Styled<D> {
        Styled::new(item, self.info)
    }

    pub fn warn<D: std::fmt::Display>(self, item: D) -> Styled<D> {
        Styled::new(item, self.warn)
    }

    pub fn error<D: std::fmt::Display>(self, item: D) -> Styled<D> {
        Styled::new(item, self.error)
    }

    pub fn hint<D: std::fmt::Display>(self, item: D) -> Styled<D> {
        Styled::new(item, self.hint)
    }

    pub fn expected<D: std::fmt::Display>(self, item: D) -> Styled<D> {
        Styled::new(item, self.expected)
    }

    pub fn actual<D: std::fmt::Display>(self, item: D) -> Styled<D> {
        Styled::new(item, self.actual)
    }
}

fn is_colored() -> bool {
    #[cfg(feature = "color")]
    {
        anstream::AutoStream::choice(&std::io::stderr()) != anstream::ColorChoice::Never
    }
    #[cfg(not(feature = "color"))]
    {
        false
    }
}

pub(crate) use anstyle::Style;

#[derive(Debug)]
pub struct Styled<D> {
    display: D,
    style: anstyle::Style,
}

impl<D: std::fmt::Display> Styled<D> {
    pub(crate) fn new(display: D, style: anstyle::Style) -> Self {
        Self { display, style }
    }
}

impl<D: std::fmt::Display> std::fmt::Display for Styled<D> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.style.render())?;
        self.display.fmt(f)?;
        write!(f, "{}", self.style.render_reset())?;
        Ok(())
    }
}
