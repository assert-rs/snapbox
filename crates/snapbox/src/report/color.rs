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
    pub fn always() -> Self {
        if cfg!(feature = "color") {
            Self {
                info: anstyle::AnsiColor::Green.into(),
                warn: anstyle::AnsiColor::Yellow.into(),
                error: anstyle::AnsiColor::Red.into(),
                hint: anstyle::Effects::DIMMED.into(),
                expected: anstyle::AnsiColor::Green | anstyle::Effects::UNDERLINE,
                actual: anstyle::AnsiColor::Red | anstyle::Effects::UNDERLINE,
            }
        } else {
            Self::never()
        }
    }

    pub fn never() -> Self {
        Self::default()
    }

    pub fn auto() -> Self {
        if is_colored() {
            Self::always()
        } else {
            Self::never()
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
        anstyle_stream::AutoStream::choice(&std::io::stderr()) != anstyle_stream::ColorChoice::Never
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
