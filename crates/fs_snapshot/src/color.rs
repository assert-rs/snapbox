#[derive(Copy, Clone, Debug, Default)]
#[allow(dead_code)]
pub(crate) struct Palette {
    pub(crate) info: styled::Style,
    pub(crate) error: styled::Style,
    pub(crate) hint: styled::Style,
}

impl Palette {
    #[cfg(feature = "color")]
    pub(crate) fn always() -> Self {
        Self {
            info: styled::Style(yansi::Style::new(yansi::Color::Green)),
            error: styled::Style(yansi::Style::new(yansi::Color::Red)),
            hint: styled::Style(yansi::Style::new(yansi::Color::Unset).dimmed()),
        }
    }

    #[cfg(not(feature = "color"))]
    pub(crate) fn always() -> Self {
        Self::never()
    }

    pub(crate) fn never() -> Self {
        Self::default()
    }

    pub(crate) fn auto() -> Self {
        if concolor::get(concolor::Stream::Either).ansi_color() {
            Self::always()
        } else {
            Self::never()
        }
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
