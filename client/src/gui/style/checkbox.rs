use crate::gui::style::{
    GOLD_400, GOLD_500, INK_500, INK_600, INK_700, TEXT_MUTED, TEXT_ON_ACCENT,
    TEXT_PRIMARY, TEXT_SECONDARY, WHITE_A06, WHITE_A24, XindelerUpdaterTheme,
};
use iced::{
    Background, Border, Color,
    widget::{checkbox, checkbox::Appearance},
};

#[derive(Default)]
pub enum CheckboxStyle {
    #[default]
    Default,
}

impl checkbox::StyleSheet for XindelerUpdaterTheme {
    type Style = CheckboxStyle;

    fn active(&self, _style: &Self::Style, is_checked: bool) -> Appearance {
        if is_checked {
            Appearance {
                background: Background::Color(GOLD_500),
                icon_color: TEXT_ON_ACCENT,
                border: Border {
                    color: GOLD_500,
                    width: 1.0,
                    radius: 4.0.into(),
                },
                text_color: Some(TEXT_PRIMARY),
            }
        } else {
            Appearance {
                background: Background::Color(INK_600),
                icon_color: TEXT_ON_ACCENT,
                border: Border {
                    color: WHITE_A24,
                    width: 1.0,
                    radius: 4.0.into(),
                },
                text_color: Some(TEXT_SECONDARY),
            }
        }
    }

    fn hovered(&self, _style: &Self::Style, is_checked: bool) -> Appearance {
        if is_checked {
            Appearance {
                background: Background::Color(GOLD_400),
                icon_color: TEXT_ON_ACCENT,
                border: Border {
                    color: GOLD_400,
                    width: 1.0,
                    radius: 4.0.into(),
                },
                text_color: Some(TEXT_PRIMARY),
            }
        } else {
            Appearance {
                background: Background::Color(INK_500),
                icon_color: TEXT_ON_ACCENT,
                border: Border {
                    color: Color::from_rgba(GOLD_500.r, GOLD_500.g, GOLD_500.b, 0.6),
                    width: 1.0,
                    radius: 4.0.into(),
                },
                text_color: Some(TEXT_PRIMARY),
            }
        }
    }

    fn disabled(&self, _style: &Self::Style, _is_checked: bool) -> Appearance {
        Appearance {
            background: Background::Color(INK_700),
            icon_color: TEXT_MUTED,
            border: Border {
                color: WHITE_A06,
                width: 1.0,
                radius: 4.0.into(),
            },
            text_color: Some(TEXT_MUTED),
        }
    }
}
