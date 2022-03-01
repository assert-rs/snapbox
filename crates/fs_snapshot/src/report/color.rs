#[derive(Copy, Clone, Debug)]
#[allow(dead_code)]
pub struct Palette {
    info: styled::Style,
    warn: styled::Style,
    error: styled::Style,
    hint: styled::Style,
}

impl Palette {
    #[cfg(feature = "color")]
    pub fn always() -> Self {
        Self {
            info: styled::Style(yansi::Style::new(yansi::Color::Green)),
            warn: styled::Style(yansi::Style::new(yansi::Color::Yellow)),
            error: styled::Style(yansi::Style::new(yansi::Color::Red)),
            hint: styled::Style(yansi::Style::new(yansi::Color::Unset).dimmed()),
        }
    }

    #[cfg(not(feature = "color"))]
    pub fn always() -> Self {
        Self::never()
    }

    pub fn never() -> Self {
        Self {
            info: Default::default(),
            warn: Default::default(),
            error: Default::default(),
            hint: Default::default(),
        }
    }

    pub fn auto() -> Self {
        if is_colored() {
            Self::always()
        } else {
            Self::never()
        }
    }

    pub fn info<D: std::fmt::Display>(&self, item: D) -> Styled<D> {
        self.info.paint(item)
    }

    pub fn warn<D: std::fmt::Display>(&self, item: D) -> Styled<D> {
        self.warn.paint(item)
    }

    pub fn error<D: std::fmt::Display>(&self, item: D) -> Styled<D> {
        self.error.paint(item)
    }

    pub fn hint<D: std::fmt::Display>(&self, item: D) -> Styled<D> {
        self.hint.paint(item)
    }
}

fn is_colored() -> bool {
    #[cfg(feature = "color")]
    {
        concolor::get(concolor::Stream::Either).ansi_color()
    }

    #[cfg(not(feature = "color"))]
    {
        false
    }
}

pub use styled::Styled;

#[cfg(feature = "color")]
mod styled {
    #[derive(Copy, Clone, Debug, Default)]
    pub(crate) struct Style(pub(crate) yansi::Style);

    impl Style {
        pub(crate) fn paint<T: std::fmt::Display>(self, item: T) -> Styled<T> {
            Styled(self.0.paint(item))
        }
    }

    pub struct Styled<D: std::fmt::Display>(yansi::Paint<D>);

    impl<D: std::fmt::Display> std::fmt::Display for Styled<D> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.0.fmt(f)
        }
    }
}

#[cfg(not(feature = "color"))]
mod styled {
    #[derive(Copy, Clone, Debug, Default)]
    pub(crate) struct Style;

    impl Style {
        pub(crate) fn paint<T: std::fmt::Display>(self, item: T) -> Styled<T> {
            Styled(item)
        }
    }

    pub struct Styled<D: std::fmt::Display>(D);

    impl<D: std::fmt::Display> std::fmt::Display for Styled<D> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.0.fmt(f)
        }
    }
}
