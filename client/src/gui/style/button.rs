use crate::gui::style::{
    ARCANE_400, ARCANE_500, ARCANE_600, CRIMSON_500, CRIMSON_600, DISCORD_BLURPLE,
    GOLD_400, GOLD_500, GOLD_700, INK_500, INK_600, INK_700, MASTODON_PURPLE, RADIUS_MD,
    RADIUS_PILL, RADIUS_SM, REDDIT_ORANGE, TEXT_MUTED, TEXT_ON_ACCENT, TEXT_PRIMARY,
    TEXT_SECONDARY, TWITCH_PURPLE, WHITE_A06, WHITE_A10, WHITE_A16, XindelerUpdaterTheme,
    YOUTUBE_RED,
};
use iced::{
    Background, Border, Color, Shadow, Vector,
    widget::{button, button::Appearance},
};

/// Intent-based button styles - see docs/design/ui-refresh-spec.md §5.1. Each variant
/// implements all four of iced's `active`/`hovered`/`pressed`/`disabled` explicitly
/// (§2.6: the previous styles left `pressed` and `disabled` to iced's defaults, which
/// is why nothing visibly reacted to a press and disabled buttons just looked washed
/// out).
#[derive(Debug, Clone, Copy, Default)]
pub enum ButtonStyle {
    /// The one action the current state is about (Play, Download, Retry, ...).
    #[default]
    Primary,
    /// A competing but subordinate action next to a Primary (Server Browser).
    Secondary,
    /// Coloured-but-not-primary - Update sitting next to Play.
    Accent,
    /// Low-emphasis text/icon action (nav links, "Not now").
    Ghost,
    /// Destructive-ish (Cancel download).
    Danger,
    /// Square icon-only control (settings gear, folder picker, help link).
    Icon,
    Chip(BrowserButtonStyle),
    ListRow(ServerListEntryButtonState),
    ColumnHeading,
    Transparent,
}

#[derive(Debug, Clone, Copy)]
pub enum BrowserButtonStyle {
    Discord,
    Mastodon,
    Reddit,
    Youtube,
    Twitch,
    /// No specific brand - a neutral chip, not a colour guess.
    Extra,
}

#[derive(Debug, Clone, Copy)]
pub enum ServerListEntryButtonState {
    Selected,
    NotSelected,
}

impl button::StyleSheet for XindelerUpdaterTheme {
    type Style = ButtonStyle;

    fn active(&self, style: &Self::Style) -> Appearance {
        match style {
            ButtonStyle::Primary => primary_active(),
            ButtonStyle::Secondary => secondary_active(),
            ButtonStyle::Accent => accent_active(),
            ButtonStyle::Ghost => ghost_active(),
            ButtonStyle::Danger => danger_active(),
            ButtonStyle::Icon => icon_active(),
            ButtonStyle::Chip(chip) => chip_active(*chip),
            ButtonStyle::ListRow(state) => list_row_active(*state),
            ButtonStyle::ColumnHeading => column_heading_active(),
            ButtonStyle::Transparent => transparent_style(),
        }
    }

    fn hovered(&self, style: &Self::Style) -> Appearance {
        match style {
            ButtonStyle::Primary => primary_hovered(),
            ButtonStyle::Secondary => secondary_hovered(),
            ButtonStyle::Accent => accent_hovered(),
            ButtonStyle::Ghost => ghost_hovered(),
            ButtonStyle::Danger => danger_hovered(),
            ButtonStyle::Icon => icon_hovered(),
            ButtonStyle::Chip(chip) => chip_hovered(*chip),
            ButtonStyle::ListRow(state) => list_row_hovered(*state),
            ButtonStyle::ColumnHeading => Appearance {
                text_color: TEXT_PRIMARY,
                ..column_heading_active()
            },
            ButtonStyle::Transparent => transparent_style(),
        }
    }

    fn pressed(&self, style: &Self::Style) -> Appearance {
        match style {
            ButtonStyle::Primary => primary_pressed(),
            ButtonStyle::Secondary => secondary_pressed(),
            ButtonStyle::Accent => accent_pressed(),
            ButtonStyle::Ghost => ghost_pressed(),
            ButtonStyle::Danger => danger_pressed(),
            ButtonStyle::Icon => icon_pressed(),
            ButtonStyle::Chip(chip) => chip_pressed(*chip),
            ButtonStyle::ListRow(state) => list_row_hovered(*state),
            ButtonStyle::ColumnHeading => self.active(style),
            ButtonStyle::Transparent => transparent_style(),
        }
    }

    fn disabled(&self, style: &Self::Style) -> Appearance {
        match style {
            ButtonStyle::Primary => primary_disabled(),
            ButtonStyle::Secondary => secondary_disabled(),
            ButtonStyle::Accent => accent_disabled(),
            ButtonStyle::Ghost => ghost_disabled(),
            ButtonStyle::Danger => danger_disabled(),
            _ => self.active(style),
        }
    }
}

// ---- Primary ---------------------------------------------------------------------

fn primary_active() -> Appearance {
    Appearance {
        background: Some(Background::Color(GOLD_500)),
        text_color: TEXT_ON_ACCENT,
        border: Border::with_radius(crate::gui::style::RADIUS_LG),
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.35),
            offset: Vector::new(0.0, 2.0),
            blur_radius: 8.0,
        },
        ..Appearance::default()
    }
}

fn primary_hovered() -> Appearance {
    Appearance {
        background: Some(Background::Color(GOLD_400)),
        text_color: TEXT_ON_ACCENT,
        border: Border::with_radius(crate::gui::style::RADIUS_LG),
        shadow: Shadow {
            color: crate::gui::style::GOLD_GLOW,
            offset: Vector::new(0.0, 4.0),
            blur_radius: 18.0,
        },
        ..Appearance::default()
    }
}

fn primary_pressed() -> Appearance {
    Appearance {
        background: Some(Background::Color(GOLD_700)),
        text_color: TEXT_ON_ACCENT,
        border: Border::with_radius(crate::gui::style::RADIUS_LG),
        ..Appearance::default()
    }
}

fn primary_disabled() -> Appearance {
    Appearance {
        background: Some(Background::Color(INK_600)),
        text_color: TEXT_MUTED,
        border: Border {
            color: WHITE_A06,
            width: 1.0,
            radius: crate::gui::style::RADIUS_LG.into(),
        },
        ..Appearance::default()
    }
}

// ---- Secondary --------------------------------------------------------------------

fn secondary_active() -> Appearance {
    Appearance {
        background: Some(Background::Color(INK_600)),
        text_color: TEXT_SECONDARY,
        border: Border {
            color: WHITE_A10,
            width: 1.0,
            radius: RADIUS_MD.into(),
        },
        ..Appearance::default()
    }
}

fn secondary_hovered() -> Appearance {
    Appearance {
        background: Some(Background::Color(INK_500)),
        text_color: TEXT_PRIMARY,
        border: Border {
            color: WHITE_A16,
            width: 1.0,
            radius: RADIUS_MD.into(),
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.30),
            offset: Vector::new(0.0, 2.0),
            blur_radius: 8.0,
        },
        ..Appearance::default()
    }
}

fn secondary_pressed() -> Appearance {
    Appearance {
        background: Some(Background::Color(INK_700)),
        text_color: TEXT_SECONDARY,
        border: Border {
            color: WHITE_A10,
            width: 1.0,
            radius: RADIUS_MD.into(),
        },
        ..Appearance::default()
    }
}

fn secondary_disabled() -> Appearance {
    Appearance {
        background: Some(Background::Color(INK_700)),
        text_color: TEXT_MUTED,
        border: Border {
            color: WHITE_A06,
            width: 1.0,
            radius: RADIUS_MD.into(),
        },
        ..Appearance::default()
    }
}

// ---- Accent -------------------------------------------------------------------

fn accent_active() -> Appearance {
    Appearance {
        background: Some(Background::Color(ARCANE_500)),
        text_color: TEXT_SECONDARY,
        border: Border::with_radius(RADIUS_MD),
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.35),
            offset: Vector::new(0.0, 2.0),
            blur_radius: 8.0,
        },
        ..Appearance::default()
    }
}

fn accent_hovered() -> Appearance {
    Appearance {
        background: Some(Background::Color(ARCANE_400)),
        text_color: TEXT_PRIMARY,
        border: Border::with_radius(RADIUS_MD),
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.40),
            offset: Vector::new(0.0, 3.0),
            blur_radius: 12.0,
        },
        ..Appearance::default()
    }
}

fn accent_pressed() -> Appearance {
    Appearance {
        background: Some(Background::Color(ARCANE_600)),
        text_color: TEXT_PRIMARY,
        border: Border::with_radius(RADIUS_MD),
        ..Appearance::default()
    }
}

fn accent_disabled() -> Appearance {
    Appearance {
        background: Some(Background::Color(INK_600)),
        text_color: TEXT_MUTED,
        border: Border {
            color: WHITE_A06,
            width: 1.0,
            radius: RADIUS_MD.into(),
        },
        ..Appearance::default()
    }
}

// ---- Ghost --------------------------------------------------------------------

fn ghost_active() -> Appearance {
    Appearance {
        background: None,
        text_color: TEXT_SECONDARY,
        border: Border::with_radius(RADIUS_MD),
        ..Appearance::default()
    }
}

fn ghost_hovered() -> Appearance {
    Appearance {
        background: Some(Background::Color(WHITE_A10)),
        text_color: TEXT_PRIMARY,
        border: Border::with_radius(RADIUS_MD),
        ..Appearance::default()
    }
}

fn ghost_pressed() -> Appearance {
    Appearance {
        background: Some(Background::Color(WHITE_A16)),
        text_color: TEXT_PRIMARY,
        border: Border::with_radius(RADIUS_MD),
        ..Appearance::default()
    }
}

fn ghost_disabled() -> Appearance {
    Appearance {
        background: None,
        text_color: TEXT_MUTED,
        border: Border::with_radius(RADIUS_MD),
        ..Appearance::default()
    }
}

// ---- Danger -------------------------------------------------------------------

fn danger_active() -> Appearance {
    Appearance {
        background: None,
        text_color: crate::gui::style::DANGER_TEXT,
        border: Border {
            color: Color::from_rgba(CRIMSON_500.r, CRIMSON_500.g, CRIMSON_500.b, 0.55),
            width: 1.0,
            radius: RADIUS_MD.into(),
        },
        ..Appearance::default()
    }
}

fn danger_hovered() -> Appearance {
    Appearance {
        background: Some(Background::Color(Color::from_rgba(
            CRIMSON_500.r,
            CRIMSON_500.g,
            CRIMSON_500.b,
            0.16,
        ))),
        text_color: crate::gui::style::DANGER_TEXT,
        border: Border {
            color: CRIMSON_500,
            width: 1.0,
            radius: RADIUS_MD.into(),
        },
        ..Appearance::default()
    }
}

fn danger_pressed() -> Appearance {
    Appearance {
        background: Some(Background::Color(Color::from_rgba(
            CRIMSON_500.r,
            CRIMSON_500.g,
            CRIMSON_500.b,
            0.28,
        ))),
        text_color: TEXT_PRIMARY,
        border: Border {
            color: CRIMSON_600,
            width: 1.0,
            radius: RADIUS_MD.into(),
        },
        ..Appearance::default()
    }
}

fn danger_disabled() -> Appearance {
    Appearance {
        background: None,
        text_color: TEXT_MUTED,
        border: Border {
            color: WHITE_A06,
            width: 1.0,
            radius: RADIUS_MD.into(),
        },
        ..Appearance::default()
    }
}

// ---- Icon ---------------------------------------------------------------------

fn icon_active() -> Appearance {
    Appearance {
        background: None,
        text_color: TEXT_SECONDARY,
        border: Border::with_radius(RADIUS_SM),
        ..Appearance::default()
    }
}

fn icon_hovered() -> Appearance {
    Appearance {
        background: Some(Background::Color(WHITE_A10)),
        text_color: TEXT_PRIMARY,
        border: Border::with_radius(RADIUS_SM),
        ..Appearance::default()
    }
}

fn icon_pressed() -> Appearance {
    Appearance {
        background: Some(Background::Color(WHITE_A16)),
        text_color: TEXT_PRIMARY,
        border: Border::with_radius(RADIUS_SM),
        ..Appearance::default()
    }
}

// ---- Column heading -------------------------------------------------------------

fn column_heading_active() -> Appearance {
    Appearance {
        background: None,
        text_color: TEXT_MUTED,
        border: Border::default(),
        ..Appearance::default()
    }
}

// ---- Transparent (used as a plain click target, e.g. the news card) -------------

fn transparent_style() -> Appearance {
    Appearance {
        background: None,
        ..Appearance::default()
    }
}

// ---- List rows (server browser) --------------------------------------------------

fn list_row_active(state: ServerListEntryButtonState) -> Appearance {
    match state {
        ServerListEntryButtonState::Selected => Appearance {
            background: Some(Background::Color(INK_600)),
            text_color: TEXT_PRIMARY,
            border: Border::with_radius(RADIUS_SM),
            ..Appearance::default()
        },
        ServerListEntryButtonState::NotSelected => Appearance {
            background: None,
            text_color: TEXT_SECONDARY,
            border: Border::with_radius(RADIUS_SM),
            ..Appearance::default()
        },
    }
}

fn list_row_hovered(state: ServerListEntryButtonState) -> Appearance {
    match state {
        ServerListEntryButtonState::Selected => Appearance {
            background: Some(Background::Color(INK_500)),
            text_color: TEXT_PRIMARY,
            border: Border::with_radius(RADIUS_SM),
            ..Appearance::default()
        },
        ServerListEntryButtonState::NotSelected => Appearance {
            background: Some(Background::Color(WHITE_A06)),
            text_color: TEXT_PRIMARY,
            border: Border::with_radius(RADIUS_SM),
            ..Appearance::default()
        },
    }
}

// ---- Brand chips ------------------------------------------------------------------

fn chip_brand_color(style: BrowserButtonStyle) -> Option<Color> {
    match style {
        BrowserButtonStyle::Discord => Some(*DISCORD_BLURPLE),
        BrowserButtonStyle::Youtube => Some(*YOUTUBE_RED),
        BrowserButtonStyle::Mastodon => Some(*MASTODON_PURPLE),
        BrowserButtonStyle::Reddit => Some(*REDDIT_ORANGE),
        BrowserButtonStyle::Twitch => Some(*TWITCH_PURPLE),
        // No real brand behind "Extra" - stays neutral, see chip_active/hovered below.
        BrowserButtonStyle::Extra => None,
    }
}

fn chip_active(style: BrowserButtonStyle) -> Appearance {
    match chip_brand_color(style) {
        Some(color) => Appearance {
            background: Some(Background::Color(color)),
            text_color: Color::WHITE,
            border: Border::with_radius(RADIUS_PILL),
            ..Appearance::default()
        },
        None => Appearance {
            background: Some(Background::Color(INK_600)),
            text_color: TEXT_SECONDARY,
            border: Border {
                color: WHITE_A10,
                width: 1.0,
                radius: RADIUS_PILL.into(),
            },
            ..Appearance::default()
        },
    }
}

fn chip_hovered(style: BrowserButtonStyle) -> Appearance {
    match chip_brand_color(style) {
        Some(color) => Appearance {
            background: Some(Background::Color(color_multiply(color, 1.08))),
            text_color: Color::WHITE,
            border: Border::with_radius(RADIUS_PILL),
            ..Appearance::default()
        },
        None => Appearance {
            background: Some(Background::Color(INK_500)),
            text_color: TEXT_PRIMARY,
            border: Border {
                color: WHITE_A16,
                width: 1.0,
                radius: RADIUS_PILL.into(),
            },
            ..Appearance::default()
        },
    }
}

fn chip_pressed(style: BrowserButtonStyle) -> Appearance {
    match chip_brand_color(style) {
        Some(color) => Appearance {
            background: Some(Background::Color(color_multiply(color, 0.92))),
            text_color: Color::WHITE,
            border: Border::with_radius(RADIUS_PILL),
            ..Appearance::default()
        },
        None => Appearance {
            background: Some(Background::Color(INK_700)),
            text_color: TEXT_SECONDARY,
            border: Border {
                color: WHITE_A10,
                width: 1.0,
                radius: RADIUS_PILL.into(),
            },
            ..Appearance::default()
        },
    }
}

fn color_multiply(color: Color, multiplier: f32) -> Color {
    Color::new(
        (color.r * multiplier).clamp(0.0, 1.0),
        (color.g * multiplier).clamp(0.0, 1.0),
        (color.b * multiplier).clamp(0.0, 1.0),
        color.a,
    )
}
