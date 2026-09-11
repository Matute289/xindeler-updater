use crate::gui::style::{WHITE_A10, XindelerUpdaterTheme};
use iced::widget::{
    rule,
    rule::{Appearance, FillMode},
};

#[derive(Debug, Clone, Copy, Default)]
pub enum RuleStyle {
    #[default]
    Default,
}

impl rule::StyleSheet for XindelerUpdaterTheme {
    type Style = RuleStyle;

    fn appearance(&self, style: &Self::Style) -> Appearance {
        match style {
            RuleStyle::Default => default_rule_style(),
        }
    }
}

fn default_rule_style() -> Appearance {
    Appearance {
        width: 1,
        color: WHITE_A10,
        radius: 0.0.into(),
        fill_mode: FillMode::Full,
    }
}
