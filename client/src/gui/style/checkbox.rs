use crate::gui::style::{
    CORNFLOWER_BLUE, LIGHT_GREY, MEDIUM_GREY, VERY_DARK_GREY, XindelerUpdaterTheme,
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
        appearance(is_checked)
    }

    fn hovered(&self, _style: &Self::Style, is_checked: bool) -> Appearance {
        appearance(is_checked)
    }
}

fn appearance(is_checked: bool) -> Appearance {
    Appearance {
        background: Background::Color(if is_checked {
            CORNFLOWER_BLUE
        } else {
            VERY_DARK_GREY
        }),
        icon_color: Color::WHITE,
        border: Border {
            color: MEDIUM_GREY,
            width: 1.0,
            radius: 4.0.into(),
        },
        text_color: Some(LIGHT_GREY),
    }
}
