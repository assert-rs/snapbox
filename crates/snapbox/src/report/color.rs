#[derive(Copy, Clone, Debug, Default)]
pub struct Palette {
    pub(crate) info: Style,
    pub(crate) warn: Style,
    pub(crate) error: Style,
    pub(crate) hint: Style,
    pub(crate) expected: Style,
    pub(crate) actual: Style,
}

impl Palette {
    pub fn color() -> Self {
        if cfg!(feature = "color") {
            Self {
                info: anstyle::AnsiColor::Green.on_default(),
                warn: anstyle::AnsiColor::Yellow.on_default(),
                error: anstyle::AnsiColor::Red.on_default(),
                hint: anstyle::Effects::DIMMED.into(),
                expected: anstyle::AnsiColor::Red.on_default() | anstyle::Effects::UNDERLINE,
                actual: anstyle::AnsiColor::Green.on_default() | anstyle::Effects::UNDERLINE,
            }
        } else {
            Self::plain()
        }
    }

    pub fn plain() -> Self {
        Self::default()
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

pub(crate) use anstyle::Style;

#[derive(Debug)]
pub struct Styled<D> {
    display: D,
    style: Style,
}

impl<D: std::fmt::Display> Styled<D> {
    pub(crate) fn new(display: D, style: Style) -> Self {
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
