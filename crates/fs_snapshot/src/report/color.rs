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

    pub fn info(&self, item: impl std::fmt::Display) -> impl std::fmt::Display {
        self.info.paint(item)
    }

    pub fn warn(&self, item: impl std::fmt::Display) -> impl std::fmt::Display {
        self.warn.paint(item)
    }

    pub fn error(&self, item: impl std::fmt::Display) -> impl std::fmt::Display {
        self.error.paint(item)
    }

    pub fn hint(&self, item: impl std::fmt::Display) -> impl std::fmt::Display {
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

#[cfg(feature = "color")]
mod styled {
    #[derive(Copy, Clone, Debug, Default)]
    pub(crate) struct Style(pub(crate) yansi::Style);

    impl Style {
        pub(crate) fn paint<T: std::fmt::Display>(self, item: T) -> impl std::fmt::Display {
            self.0.paint(item)
        }
    }
}

#[cfg(not(feature = "color"))]
mod styled {
    #[derive(Copy, Clone, Debug, Default)]
    pub(crate) struct Style;

    impl Style {
        pub(crate) fn paint<T: std::fmt::Display>(self, item: T) -> impl std::fmt::Display {
            item
        }
    }
}
