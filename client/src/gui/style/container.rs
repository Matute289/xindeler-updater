use crate::gui::style::{
    GOLD_400, GOLD_500, INK_600, INK_700, INK_800, PANEL_VEIL, RADIUS_LG, RADIUS_MD,
    RADIUS_PILL, RADIUS_XL, SCRIM, SHADOW_KEY, TEXT_MUTED, TEXT_PRIMARY, TEXT_SECONDARY,
    WHITE_A06, WHITE_A10, XindelerUpdaterTheme,
};
use iced::{
    Background, Border, Color, Shadow, Vector,
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
    /// A card-shaped panel that sits on an already-dark surface (auto-update options,
    /// error callouts) - see docs/design/ui-refresh-spec.md §5.3/§6.3.
    Card,
    /// An inline error callout: tinted crimson background/border, used for raw error
    /// text instead of a bare paragraph - see §6.3.
    ErrorCallout,
    /// A small solid-colour dot (modal eyebrow, toast icon) - carries its own colour
    /// since it's reused with gold/arcane/crimson/success depending on context.
    StatusDot(Color),
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
            ContainerStyle::Card => card_container_style(),
            ContainerStyle::ErrorCallout => error_callout_container_style(),
            ContainerStyle::StatusDot(color) => status_dot_container_style(*color),
        }
    }
}

fn dark_container_style() -> Appearance {
    // A "veil" rather than a flat fill, so a hint of the rotating background art shows
    // through the long-lived opaque panels - a cheap depth cue (spec §9).
    Appearance {
        background: Some(Background::Color(PANEL_VEIL)),
        text_color: Some(TEXT_PRIMARY),
        ..Appearance::default()
    }
}

fn announcement_container_style() -> Appearance {
    Appearance {
        background: Some(Background::Color(Color::from_rgba(
            GOLD_500.r, GOLD_500.g, GOLD_500.b, 0.14,
        ))),
        text_color: Some(GOLD_400),
        border: Border {
            color: Color::from_rgba(GOLD_500.r, GOLD_500.g, GOLD_500.b, 0.30),
            width: 1.0,
            radius: RADIUS_MD.into(),
        },
        ..Appearance::default()
    }
}

fn loading_blogpost_container_style() -> Appearance {
    Appearance {
        background: Some(Background::Color(INK_800)),
        border: Border {
            color: WHITE_A06,
            width: 1.0,
            ..Default::default()
        },
        text_color: Some(TEXT_MUTED),
        ..Appearance::default()
    }
}

fn blogpost_container_style() -> Appearance {
    Appearance {
        background: Some(Background::Color(INK_700)),
        text_color: Some(TEXT_PRIMARY),
        border: Border {
            color: WHITE_A06,
            width: 1.0,
            // Square top corners: the post image sits above this container and iced
            // can't clip it to a rounded corner (spec §2.8).
            radius: [0.0, 0.0, RADIUS_LG, RADIUS_LG].into(),
        },
        ..Appearance::default()
    }
}

fn column_heading_container_style() -> Appearance {
    Appearance {
        text_color: Some(TEXT_MUTED),
        ..Appearance::default()
    }
}

fn changelog_header_container_style() -> Appearance {
    Appearance {
        background: Some(Background::Color(INK_800)),
        text_color: Some(TEXT_PRIMARY),
        ..Appearance::default()
    }
}

fn extra_browser_container_style() -> Appearance {
    Appearance {
        background: Some(Background::Color(INK_600)),
        text_color: Some(TEXT_SECONDARY),
        border: Border {
            color: WHITE_A10,
            width: 1.0,
            radius: RADIUS_PILL.into(),
        },
        ..Appearance::default()
    }
}

fn modal_backdrop_container_style() -> Appearance {
    // No blur is possible in iced 0.12 (spec §2.8) - a higher opacity compensates.
    Appearance {
        background: Some(Background::Color(SCRIM)),
        ..Appearance::default()
    }
}

fn modal_dialog_container_style() -> Appearance {
    Appearance {
        background: Some(Background::Color(INK_700)),
        text_color: Some(TEXT_PRIMARY),
        border: Border {
            color: WHITE_A10,
            width: 1.0,
            radius: RADIUS_XL.into(),
        },
        shadow: Shadow {
            color: SHADOW_KEY,
            offset: Vector::new(0.0, 16.0),
            blur_radius: 40.0,
        },
    }
}

fn toast_container_style() -> Appearance {
    Appearance {
        background: Some(Background::Color(INK_700)),
        text_color: Some(TEXT_PRIMARY),
        border: Border {
            color: WHITE_A10,
            width: 1.0,
            radius: RADIUS_MD.into(),
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.45),
            offset: Vector::new(0.0, 4.0),
            blur_radius: 16.0,
        },
    }
}

fn tooltip_container_style() -> Appearance {
    Appearance {
        text_color: Some(TEXT_SECONDARY),
        background: Some(Background::Color(INK_700)),
        border: Border {
            color: WHITE_A10,
            width: 1.0,
            radius: RADIUS_MD.into(),
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.45),
            offset: Vector::new(0.0, 4.0),
            blur_radius: 12.0,
        },
    }
}

fn card_container_style() -> Appearance {
    Appearance {
        background: Some(Background::Color(INK_700)),
        text_color: Some(TEXT_PRIMARY),
        border: Border {
            color: WHITE_A06,
            width: 1.0,
            radius: RADIUS_LG.into(),
        },
        ..Appearance::default()
    }
}

fn status_dot_container_style(color: Color) -> Appearance {
    Appearance {
        background: Some(Background::Color(color)),
        border: Border::with_radius(4.0),
        ..Appearance::default()
    }
}

fn error_callout_container_style() -> Appearance {
    let crimson = crate::gui::style::CRIMSON_500;
    Appearance {
        background: Some(Background::Color(Color::from_rgba(
            crimson.r, crimson.g, crimson.b, 0.12,
        ))),
        text_color: Some(crate::gui::style::DANGER_TEXT),
        border: Border {
            color: Color::from_rgba(crimson.r, crimson.g, crimson.b, 0.35),
            width: 1.0,
            radius: RADIUS_MD.into(),
        },
        ..Appearance::default()
    }
}
