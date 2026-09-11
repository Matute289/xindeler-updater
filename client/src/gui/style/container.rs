use crate::gui::style::{
    ALMOST_BLACK, BLOG_POST_BACKGROUND_BLUE, BRIGHT_ORANGE, DARK_WHITE, LIGHT_GREY,
    LIME_GREEN, MEDIUM_GREY, NAVY_BLUE, VERY_DARK_GREY, XindelerUpdaterTheme,
};
use iced::{
    Background, Border, Color,
    widget::{container, container::Appearance},
};

#[derive(Default)]
pub enum ContainerStyle {
    #[default]
    Default,
    Dark,
    Announcement,
    LoadingBlogPost,
    BlogPost,
    ColumnHeading,
    ChangelogHeader,
    Tooltip,
    ExtraBrowser,
    ModalBackdrop,
    ModalDialog,
    Toast,
}

impl container::StyleSheet for XindelerUpdaterTheme {
    type Style = ContainerStyle;

    fn appearance(&self, style: &Self::Style) -> Appearance {
        match style {
            ContainerStyle::Default => Appearance::default(),
            ContainerStyle::Announcement => announcement_container_style(),
            ContainerStyle::Dark => dark_container_style(),
            ContainerStyle::LoadingBlogPost => loading_blogpost_container_style(),
            ContainerStyle::BlogPost => blogpost_container_style(),
            ContainerStyle::ColumnHeading => column_heading_container_style(),
            ContainerStyle::ChangelogHeader => changelog_header_container_style(),
            ContainerStyle::Tooltip => tooltip_container_style(),
            ContainerStyle::ExtraBrowser => extra_browser_container_style(),
            ContainerStyle::ModalBackdrop => modal_backdrop_container_style(),
            ContainerStyle::ModalDialog => modal_dialog_container_style(),
            ContainerStyle::Toast => toast_container_style(),
        }
    }
}

fn dark_container_style() -> Appearance {
    Appearance {
        background: Some(Background::Color(VERY_DARK_GREY)),
        text_color: Some(Color::WHITE),
        ..Appearance::default()
    }
}

fn announcement_container_style() -> Appearance {
    Appearance {
        background: Some(Background::Color(BRIGHT_ORANGE)),
        text_color: Some(Color::WHITE),
        ..Appearance::default()
    }
}

fn loading_blogpost_container_style() -> Appearance {
    Appearance {
        background: None,
        border: Border {
            color: DARK_WHITE,
            width: 0.7,
            ..Default::default()
        },
        text_color: Some(DARK_WHITE),
        ..Appearance::default()
    }
}

fn blogpost_container_style() -> Appearance {
    Appearance {
        background: Some(Background::Color(BLOG_POST_BACKGROUND_BLUE)),
        text_color: Some(Color::WHITE),
        ..Appearance::default()
    }
}

fn column_heading_container_style() -> Appearance {
    Appearance {
        text_color: Some(Color::WHITE),
        ..Appearance::default()
    }
}

fn changelog_header_container_style() -> Appearance {
    Appearance {
        background: Some(Background::Color(Color::BLACK)),
        text_color: Some(Color::WHITE),
        ..Appearance::default()
    }
}

fn extra_browser_container_style() -> Appearance {
    Appearance {
        background: Some(Background::Color(LIME_GREEN)),
        border: Border::with_radius(25.0),
        ..Appearance::default()
    }
}

fn modal_backdrop_container_style() -> Appearance {
    Appearance {
        background: Some(Background::Color(ALMOST_BLACK)),
        ..Appearance::default()
    }
}

fn modal_dialog_container_style() -> Appearance {
    Appearance {
        background: Some(Background::Color(VERY_DARK_GREY)),
        text_color: Some(Color::WHITE),
        border: Border {
            color: MEDIUM_GREY,
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Appearance::default()
    }
}

fn toast_container_style() -> Appearance {
    Appearance {
        background: Some(Background::Color(LIME_GREEN)),
        text_color: Some(Color::WHITE),
        border: Border::with_radius(4.0),
        ..Appearance::default()
    }
}

fn tooltip_container_style() -> Appearance {
    Appearance {
        text_color: Some(LIGHT_GREY),
        background: Some(Background::Color(NAVY_BLUE)),
        border: Border {
            color: MEDIUM_GREY,
            width: 1.0,
            ..Default::default()
        },
        ..Appearance::default()
    }
}
