use crate::gui::style::{
    INK_500, INK_600, RADIUS_MD, TEXT_MUTED, TEXT_PRIMARY, TEXT_SECONDARY, WHITE_A10,
    WHITE_A16, XindelerUpdaterTheme,
};
use iced::{
    Background, Border,
    widget::{
        pick_list,
        pick_list::{Appearance, StyleSheet},
    },
};

#[derive(Copy, Clone, Debug, Default)]
pub enum PickListStyle {
    #[default]
    Default,
}

impl pick_list::StyleSheet for XindelerUpdaterTheme {
    type Style = PickListStyle;

    fn active(&self, _: &<Self as StyleSheet>::Style) -> Appearance {
        Appearance {
            text_color: TEXT_PRIMARY,
            background: Background::Color(INK_600),
            // icon_size: 0.5, TODO: no longer settable in this iced version - see
            // docs/design/ui-refresh-spec.md §2.8.
            border: Border {
                width: 1.0,
                radius: RADIUS_MD.into(),
                color: WHITE_A10,
            },
            handle_color: TEXT_SECONDARY,
            placeholder_color: TEXT_MUTED,
        }
    }

    fn hovered(&self, _: &<Self as StyleSheet>::Style) -> Appearance {
        Appearance {
            text_color: TEXT_PRIMARY,
            background: Background::Color(INK_500),
            border: Border {
                width: 1.0,
                radius: RADIUS_MD.into(),
                color: WHITE_A16,
            },
            handle_color: TEXT_PRIMARY,
            placeholder_color: TEXT_MUTED,
        }
    }
}
