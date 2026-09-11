use crate::gui::style::{
    DANGER_TEXT, GOLD_400, SUCCESS_TEXT, TEXT_MUTED, TEXT_PRIMARY, TEXT_SECONDARY,
    XindelerUpdaterTheme,
};
use iced::widget::{text, text::Appearance};

#[derive(Debug, Clone, Copy, Default)]
pub enum TextStyle {
    /// Inherits whatever colour the surrounding widget sets (button label, container
    /// text, ...) rather than forcing white - see docs/design/ui-refresh-spec.md §2.7.
    #[default]
    Normal,
    Primary,
    Secondary,
    Muted,
    Accent,
    Danger,
    Success,
}

impl text::StyleSheet for XindelerUpdaterTheme {
    type Style = TextStyle;

    fn appearance(&self, style: Self::Style) -> Appearance {
        match style {
            TextStyle::Normal => Appearance { color: None },
            TextStyle::Primary => text_appearance(TEXT_PRIMARY),
            TextStyle::Secondary => text_appearance(TEXT_SECONDARY),
            TextStyle::Muted => text_appearance(TEXT_MUTED),
            TextStyle::Accent => text_appearance(GOLD_400),
            TextStyle::Danger => text_appearance(DANGER_TEXT),
            TextStyle::Success => text_appearance(SUCCESS_TEXT),
        }
    }
}

fn text_appearance(color: iced::Color) -> Appearance {
    Appearance { color: Some(color) }
}
