use crate::gui::style::{WHITE_A06, WHITE_A16, WHITE_A24, XindelerUpdaterTheme};
use iced::{
    Background, Border, Color,
    widget::{
        container, scrollable,
        scrollable::{Appearance, Scrollbar, Scroller},
    },
};

#[derive(Default)]
pub enum ScrollableStyle {
    #[default]
    Default,
}

impl scrollable::StyleSheet for XindelerUpdaterTheme {
    type Style = ScrollableStyle;

    fn active(&self, _: &Self::Style) -> Appearance {
        Appearance {
            container: container::Appearance::default(),
            scrollbar: Scrollbar {
                background: Some(Background::Color(Color::TRANSPARENT)),
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 0.0.into(),
                },
                scroller: Scroller {
                    border: Border {
                        color: WHITE_A16,
                        width: 0.0,
                        radius: 4.0.into(),
                    },
                    color: WHITE_A16,
                },
            },
            gap: None,
        }
    }

    fn hovered(&self, _: &Self::Style, _: bool) -> Appearance {
        Appearance {
            container: container::Appearance::default(),
            scrollbar: Scrollbar {
                background: Some(Background::Color(WHITE_A06)),
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 0.0.into(),
                },
                scroller: Scroller {
                    border: Border {
                        color: WHITE_A24,
                        width: 0.0,
                        radius: 4.0.into(),
                    },
                    color: WHITE_A24,
                },
            },
            gap: None,
        }
    }
}
