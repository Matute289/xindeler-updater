use crate::gui::style::{
    INK_600, INK_700, RADIUS_MD, TEXT_PRIMARY, WHITE_A10, XindelerUpdaterTheme,
    pick_list::PickListStyle,
};
use iced::{Background, Border, overlay, overlay::menu::Appearance};

#[derive(Copy, Clone, Debug, Default)]
pub enum MenuStyle {
    #[default]
    Default,
}

impl From<PickListStyle> for MenuStyle {
    fn from(_: PickListStyle) -> Self {
        MenuStyle::Default
    }
}

impl overlay::menu::StyleSheet for XindelerUpdaterTheme {
    type Style = MenuStyle;

    fn appearance(&self, _: &Self::Style) -> Appearance {
        Appearance {
            text_color: TEXT_PRIMARY,
            background: Background::Color(INK_700),
            selected_background: Background::Color(INK_600),
            selected_text_color: TEXT_PRIMARY,
            border: Border {
                color: WHITE_A10,
                width: 1.0,
                radius: RADIUS_MD.into(),
            },
        }
    }
}
