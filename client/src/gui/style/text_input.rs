use crate::gui::style::{
    GOLD_500, INK_600, INK_700, RADIUS_MD, TEXT_MUTED, TEXT_PRIMARY, WHITE_A10,
    XindelerUpdaterTheme,
};
use iced::{
    Background, Border, Color,
    widget::{text_input, text_input::Appearance},
};

#[derive(Default)]
pub enum TextInputStyle {
    #[default]
    Default,
}

impl text_input::StyleSheet for XindelerUpdaterTheme {
    type Style = TextInputStyle;

    fn active(&self, _: &Self::Style) -> Appearance {
        Appearance {
            background: Background::Color(INK_600),
            border: Border {
                color: WHITE_A10,
                width: 1.0,
                radius: RADIUS_MD.into(),
            },
            icon_color: Default::default(),
        }
    }

    fn focused(&self, _: &Self::Style) -> Appearance {
        Appearance {
            background: Background::Color(INK_600),
            border: Border {
                color: GOLD_500,
                width: 1.0,
                radius: RADIUS_MD.into(),
            },
            icon_color: Default::default(),
        }
    }

    fn placeholder_color(&self, _: &Self::Style) -> Color {
        TEXT_MUTED
    }

    fn value_color(&self, _: &Self::Style) -> Color {
        TEXT_PRIMARY
    }

    fn selection_color(&self, _: &Self::Style) -> Color {
        Color::from_rgba(GOLD_500.r, GOLD_500.g, GOLD_500.b, 0.35)
    }

    fn disabled_color(&self, _style: &Self::Style) -> Color {
        TEXT_MUTED
    }

    fn disabled(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            background: Background::Color(INK_700),
            border: Border {
                color: WHITE_A10,
                width: 1.0,
                radius: RADIUS_MD.into(),
            },
            icon_color: Default::default(),
        }
    }
}
