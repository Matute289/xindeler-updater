use iced::{Color, application, application::Appearance};
use lazy_static::lazy_static;

pub mod button;
pub mod checkbox;
pub mod container;
pub mod menu;
pub mod pick_list;
pub mod progress_bar;
pub mod rule;
pub mod scrollable;
pub mod text;
pub mod text_input;

// Colour system - see docs/design/ui-refresh-spec.md §3. Derived from the shipped key
// art (indigo-black backgrounds, gold/amber logo) rather than picked per-widget.

// Surfaces - a tonal ramp so panels/cards/modals read as stacked layers instead of one
// flat background.
pub const INK_900: Color = rgb8(11, 12, 18);
pub const INK_800: Color = rgb8(18, 20, 28);
pub const INK_700: Color = rgb8(25, 28, 39);
pub const INK_600: Color = rgb8(34, 38, 52);
pub const INK_500: Color = rgb8(46, 51, 70);

// Translucent helpers.
pub const SCRIM: Color = rgba8(11, 12, 18, 0.78);
pub const PANEL_VEIL: Color = rgba8(18, 20, 28, 0.88);
pub const WHITE_A06: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.06);
pub const WHITE_A10: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.10);
pub const WHITE_A16: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.16);
pub const WHITE_A24: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.24);
pub const SHADOW_KEY: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.55);

// Text.
pub const TEXT_PRIMARY: Color = rgb8(242, 243, 247);
pub const TEXT_SECONDARY: Color = rgb8(168, 174, 194);
pub const TEXT_MUTED: Color = rgb8(134, 142, 163);
pub const TEXT_ON_ACCENT: Color = rgb8(11, 12, 18);

// Accents - one primary (gold), one secondary (arcane), one danger (crimson). Each is a
// four-stop ramp: 400 hover (lighter, this is dark mode), 500 active/resting, 600
// border/selected, 700/600 pressed (a deliberate multi-step drop, not a nuance).
pub const GOLD_400: Color = rgb8(242, 180, 87);
pub const GOLD_500: Color = rgb8(232, 163, 61);
pub const GOLD_600: Color = rgb8(201, 134, 42);
pub const GOLD_700: Color = rgb8(176, 115, 31);
pub const GOLD_GLOW: Color = rgba8(232, 163, 61, 0.35);
pub const ARCANE_400: Color = rgb8(110, 96, 226);
pub const ARCANE_500: Color = rgb8(99, 85, 216);
pub const ARCANE_600: Color = rgb8(83, 70, 188);
pub const CRIMSON_400: Color = rgb8(204, 62, 67);
pub const CRIMSON_500: Color = rgb8(192, 56, 60);
pub const CRIMSON_600: Color = rgb8(163, 47, 51);
pub const DANGER_TEXT: Color = rgb8(242, 119, 122);
pub const SUCCESS_TEXT: Color = rgb8(62, 207, 142);

// Shape and spacing tokens. Deliberately tighter than the 8/12/16 web consensus -
// launchers (Steam 2px, itch.io 3px, Fluent's control default 4px) don't follow the
// rounded-corner wave, and a voxel game's UI shouldn't look rounder than its art.
pub const RADIUS_SM: f32 = 4.0;
pub const RADIUS_MD: f32 = 6.0;
pub const RADIUS_LG: f32 = 10.0;
pub const RADIUS_XL: f32 = 14.0;
pub const RADIUS_PILL: f32 = 999.0;
pub const BORDER_HAIRLINE: f32 = 1.0;

const fn rgb8(red: u8, green: u8, blue: u8) -> Color {
    Color::from_rgb(
        red as f32 / 255.0,
        green as f32 / 255.0,
        blue as f32 / 255.0,
    )
}

const fn rgba8(red: u8, green: u8, blue: u8, alpha: f32) -> Color {
    Color::from_rgba(
        red as f32 / 255.0,
        green as f32 / 255.0,
        blue as f32 / 255.0,
        alpha,
    )
}

lazy_static! {
    static ref DISCORD_BLURPLE: Color = rgb8(88, 101, 242);
    static ref MASTODON_PURPLE: Color = rgb8(99, 100, 255);
    static ref REDDIT_ORANGE: Color = rgb8(255, 69, 0);
    static ref YOUTUBE_RED: Color = rgb8(255, 0, 0);
    static ref TWITCH_PURPLE: Color = rgb8(100, 65, 165);
}

#[derive(Default)]
pub struct XindelerUpdaterTheme {}

#[derive(Default)]
pub enum XindelerUpdaterThemeStyle {
    #[default]
    Default,
}

impl application::StyleSheet for XindelerUpdaterTheme {
    type Style = XindelerUpdaterThemeStyle;

    fn appearance(&self, _: &Self::Style) -> Appearance {
        Appearance {
            background_color: INK_900,
            text_color: TEXT_PRIMARY,
        }
    }
}
