use crate::gui::style::{GOLD_500, INK_600, RADIUS_SM, XindelerUpdaterTheme};
use iced::{
    Background,
    widget::{progress_bar, progress_bar::Appearance},
};

#[derive(Default)]
pub enum ProgressBarStyle {
    #[default]
    Default,
}

impl progress_bar::StyleSheet for XindelerUpdaterTheme {
    type Style = ProgressBarStyle;

    fn appearance(&self, style: &Self::Style) -> Appearance {
        match style {
            ProgressBarStyle::Default => default_progress_bar_style(),
        }
    }
}

fn default_progress_bar_style() -> Appearance {
    Appearance {
        background: Background::Color(INK_600),
        bar: Background::Color(GOLD_500),
        border_radius: RADIUS_SM.into(),
    }
}
