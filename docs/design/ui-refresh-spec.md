# Xindeler Updater — UI Refresh Spec

Status: proposal, ready to implement
Scope: full visual pass over the launcher (buttons, modals, toasts/status messages, settings panel, all side panels)
Target: `client/src/gui/**`, iced `0.12.1`, no new GUI dependency required (one optional Cargo feature flag, see §2.9)
Non-goal: behaviour changes. Every state machine in `game_panel.rs` / `default.rs` stays exactly as it is.

---

## 1. Research summary

**What the launcher looks like today.** The palette in `client/src/gui/style/mod.rs` is a set of one-off CSS-name constants (`LIME_GREEN`, `CORNFLOWER_BLUE`, `TOMATO_RED`, `BRIGHT_ORANGE`, `NAVY_BLUE`, `SLATE`, …) that were picked per widget rather than as a system. The result is five unrelated hues fighting on one screen: a lime Play button, a cornflower Update button, an orange announcement bar, a blue news card, a navy pick-list, and a lime progress bar — all sitting on top of key art that is almost entirely dark indigo. Buttons are flat solid fills with a 4 px radius, hover is a naive `color * 1.1` multiply, there is no pressed state, no focus treatment, and the disabled state falls through to iced's default (alpha × 0.5), which reads as "broken" rather than "not available yet". This is the specific thing that makes the app read as home-made: not the layout, which is fine, but the absence of a colour system and of state feedback.

**Where the palette should come from.** Quantising the shipped brand assets gives an unambiguous answer. `assets/images/xindeler-logo.png` is gold/amber on near-black — dominant stops `#DF9A26`, `#F7D057`, `#996525` over `#0D0E0E`. Every background in `assets/images/backgrounds/` is a dark, desaturated indigo/plum: `#1B1D30`, `#1C1E2F`, `#2D2744`, `#302846`. So the game's own art already dictates the scheme the current UI ignores: **indigo-black surfaces + one warm gold accent**.

That is also the shape of every launcher and desktop app worth copying right now, and the reference values below were pulled from the products' own production CSS rather than from trend articles. Steam ships a class literally named `btn_green_steamui`: a `#75B022 → #588A1B` gradient, `border-radius: 2px`, label `#D2EFA9` that snaps to pure white on hover while the gradient lightens to `#8ED629 → #6AA621`. GOG's tokens are the most complete public set: primary body `linear-gradient(180deg, #B5E63C, #8CCA27)`, a *separate* border gradient, `--c-button-primary-content: #212121` (black label on green), and `pressed` defined as the gradient collapsing to its own darker stop `#8CCA27`. itch.io: `#FF2449`, `border-radius: 3px`, hover `#FF2E51`, active `#E1193B` plus `top: 1px` — the button physically sinks. Spotify's Encore tokens: surfaces `#121212 / #1F1F1F / #2A2A2A`, accent `#1ED760` (**not** the decade-old `#1DB954` — that appears 6 times in their CSS versus 49 for `#1ED760`), label black, and hover implemented as `transform: scale(1.04)` rather than a colour change. Battle.net has **no hardcoded greys at all** — every surface is a white/black alpha overlay (`#ffffff08`, `#ffffff0f`, `#ffffff1f`, …) over the background, with `#148EFF` as the single brand accent. Discord's blurple `#5865F2` and surfaces `#313338 / #2B2D31 / #1E1F22` derive from its real `--brand-500` / `--primary-*` HSL variables.

Three things generalise from that evidence. (1) **One accent, everything else neutral.** Spotify, Discord, Linear, Epic and Battle.net each have exactly one. (2) **Surfaces built from alpha overlays, not fixed greys** (Battle.net, Spotify's `--background-tinted-*`, Radix's alpha scales). For a launcher that renders rotating key art behind its panels — exactly our case — this is not a preference, it is the only approach that survives an arbitrary background. (3) **In dark mode, hover means *lighter*, not darker.** Steam, Spotify, Fluent 2 and Radix all agree; Radix's dark scales even invert the light-mode relationship (`grass9 #46A758` → `grass10 #53B365`). A global `darken()` on hover is the single most common dark-mode bug.

**Trends applied here.** (1) *A tonal surface ramp* instead of one flat `VERY_DARK_GREY`, so panels, cards, inputs and modals read as stacked layers (Radix formalises it: 1–5 surfaces, 6–8 borders, 9 solid accent, 11–12 text). (2) *Real interaction states with two simultaneous signals.* Material 3's state-layer opacities are **8 % hover / 12 % focus / 12 % pressed** (the 8/10/10 figures often quoted are Material 2), and Fluent 2 shows that pressed should be a *jump*, not a nuance — its dark theme goes `brand[70]` active → `brand[80]` hover → `brand[40]` pressed, three whole steps down. This spec therefore uses a four-stop accent ramp with a deliberately deep pressed value, plus a shadow that appears on hover and disappears on press, and (borrowed from Steam) a label that brightens on hover — two signals per state, never one. (3) *Typographic hierarchy in status areas* — a small uppercase caption, one large number, muted supporting text — replacing today's row of same-size 12 px strings.

**The one place this spec deliberately does not follow the web.** Game launchers did *not* join the rounded-corner wave: Steam is at 2 px, itch.io at 3 px, Fluent 2's control default is 4 px, while Spotify and Material 3 filled buttons are full pills and shadcn sits at 10 px. A launcher styled like Linear reads as a productivity tool, not as the front door to a game — and Xindeler is a **voxel** game, where everything in the art is cubic. So the radii here are tight (4–6 px controls, 10 px hero CTA, 14 px dialogs) rather than the 8–16 px web consensus, and pills are reserved for chips and avatars. The current 25 px radius on rectangular buttons and 4 px on a 75 px-tall Play button are still both wrong, just in opposite directions.

One more useful datum: **Steam, itch.io and Battle.net's own primary buttons fail WCAG AA** ([calculated] white on `#6FA720` = 2.91:1, white on `#FF2449` = 3.75:1, white on `#148EFF` = 3.31:1). Every pairing in this spec passes. That is a genuinely cheap way to be better than the genre average.

*Sources — production CSS (primary):* [Steam `buttons.css`](https://store.fastly.steamstatic.com/public/shared/css/buttons.css) · [GOG tokens](https://www.gog.com/en) · [itch.io `main.css`](https://static.itch.io/main.css) · [Spotify Encore web-player CSS](https://open.spotifycdn.com/cdn/build/web-player/) · [Blizzard/Battle.net CSS](https://www.blizzard.com/en-us/)
*Sources — design-system repos and docs:* [Material 3 tokens (`_md-sys-state`, `_md-sys-shape`)](https://github.com/material-components/material-web/tree/main/tokens/versions/v0_192) · [Fluent 2 tokens](https://github.com/microsoft/fluentui/tree/master/packages/tokens/src) · [Radix Colors `dark.ts`](https://github.com/radix-ui/colors/blob/main/src/dark.ts) · [Radix — understanding the scale](https://www.radix-ui.com/colors/docs/palette-composition/understanding-the-scale) · [Tailwind v4 border-radius](https://tailwindcss.com/docs/border-radius) · [APCA in a nutshell](https://git.apcacontrast.com/documentation/APCA_in_a_Nutshell.html) · [WCAG 1.4.11 non-text contrast](https://dequeuniversity.com/resources/wcag2.1/1.4.11-non-text-contrast)
*Caveats:* Discord's post-refresh button radius and Linear's accent hex could **not** be verified from their own CSS (auth-gated bundles; `#5E6AD2` does not appear in linear.app's stylesheet — only `#08090A` does). Sites publishing "design tokens" for these products are automated reverse-engineering, not documentation. Epic's launcher rebuild is announced but has no published visual tokens.

---

## 2. Confirmed technical constraints (iced 0.12.1)

All of these were verified against the vendored sources in `~/.cargo/registry/src/index.crates.io-*/`: `iced-0.12.1`, `iced_core-0.12.3`, `iced_widget-0.12.3`, `iced_style-0.12.1`, `iced_wgpu-0.12.1`, `iced_tiny_skia-0.12.1`. No `iced_aw` (or any other widget crate) is in `Cargo.lock`; everything below must be built from iced core primitives.

### 2.1 Gradients — supported
`iced_core::Background` has a `Gradient(Gradient::Linear)` variant. `Linear` takes an angle in radians plus up to **8** colour stops (`add_stop(offset, color)`; stops beyond 8 or outside `0.0..=1.0` are silently dropped). Both renderers implement it: `iced_wgpu` has a dedicated gradient quad pipeline (`src/shader/quad/gradient.wgsl`), `iced_tiny_skia` maps it to `tiny_skia::LinearGradient`. Radial/conic are **not** implemented (the enum has only `Linear`).

### 2.2 Gradient + shadow are mutually exclusive on wgpu — **important**
`iced_wgpu`'s solid quad shader carries `shadow_color` / `shadow_offset` / `shadow_blur_radius`; the **gradient** quad shader carries only `border_radius`, `border_width`, `border_color`. A widget whose background is a `Gradient` therefore renders **with its shadow silently dropped** on the default wgpu backend (the tiny-skia fallback happens to draw both, so this would also look different between backends). **Decision: do not use gradients on any widget that also wants a shadow.** This spec uses flat fills + shadows everywhere, and reserves gradients for one optional non-shadowed element (§5.1 note).

### 2.3 Border radius — per-corner, fully supported
`Border::radius` is a `Radius([f32; 4])` in the order **top-left, top-right, bottom-right, bottom-left**, constructible from `f32` (all four) or `[f32; 4]`. Both shaders clamp each corner to `min(width, height) * 0.5`, so `999.0` is a safe way to express "pill". Per-corner radii are available where we need them (e.g. a card whose image has square top corners).

### 2.4 Shadows — real blurred shadows, and `shadow_offset` is dead
`iced_core::Shadow { color, offset: Vector, blur_radius: f32 }` is a genuine CSS-like drop shadow (offset + gaussian-ish blur via a rounded-box SDF in both backends), *not* a hard offset copy. It is exposed on `button::Appearance.shadow` and `container::Appearance.shadow`.

Caveats:
- **`button::Appearance.shadow_offset` (the `Vector` field the current code sets in `disabled_download_button_style`) is not read by the 0.12 button widget at all.** `iced_widget::button::draw` only passes `styling.border` and `styling.shadow` into the quad; `shadow_offset` is a leftover from 0.9. Delete every use of it.
- A quad is only drawn when `background.is_some() || border.width > 0.0 || shadow.color.a > 0.0`, so a shadow on a transparent-background widget still renders.
- **`checkbox::Appearance` has no `shadow` field** (only `background`, `icon_color`, `border`, `text_color`), and the checkbox quad is drawn with `..Quad::default()`, so checkboxes cannot have shadows.
- **`progress_bar::Appearance` has no border/shadow at all** — only `background`, `bar`, `border_radius`. No inner border, no glow on the bar.
- Shadows are *outside* the widget bounds; iced does not reserve layout space for them. Keep `blur_radius` modest (≤ 24) and make sure a shadowed widget is not flush against a clipping parent.

### 2.5 No transitions, no animation between states
Style resolution is per-frame and instantaneous: `button::draw` picks `disabled()` / `pressed()` / `hovered()` / `active()` based on the current cursor + press state, and there is no interpolation machinery anywhere in `iced_style` or `iced_widget` 0.12. **CSS-like `transition: 150ms` has no equivalent.** The only way to animate is to drive it yourself — keep a `f32` in the component state, tick it with `iced::time::every(...)` in a subscription, and feed it into a per-frame style — which costs a redraw every tick for the whole window. This spec explicitly **does not** use animation; the design is built to look finished while being instantaneous (this is also what Steam/Epic do for button state changes, which are ~0 ms or near it).

### 2.6 The four button states that actually exist
`button::StyleSheet` has `active`, `hovered`, `pressed`, `disabled`. The current `impl` in `style/button.rs` only overrides `active` and `hovered`, so:
- **pressed** falls back to the default impl = `active()` with a zeroed `shadow_offset` → visually identical to active. There is currently *no* press feedback anywhere in the app.
- **disabled** falls back to the default impl = `active()` with every colour's alpha halved. That is why disabled buttons look washed out rather than designed. Note this also means the `ButtonState::Disabled` enum variants are being *double*-dimmed: `active()` returns the slate style, then iced halves its alpha on top.

**Every style in the new system must implement all four methods explicitly.**

### 2.7 Text colour inside buttons is currently overridden — must be fixed first
`iced_core::widget::text` draws with `appearance.color.unwrap_or(style.text_color)`, and `button::draw` passes its `Appearance.text_color` down as the inherited `style.text_color`. But this app's `TextStyle::Normal` (the `#[default]`) returns `Some(Color::WHITE)`, so **every `text(...)` inside a button ignores the button's `text_color` and renders white**. That blocks the whole "dark label on a bright accent" pattern.

Fix (prerequisite for §5): make `TextStyle::Normal` return `Appearance { color: None }`. Inherited colour is already white at the application level (`application::Appearance.text_color`), so nothing else changes visually — but button labels, and any other inherited-colour context, start working.

### 2.8 Other per-widget limits worth knowing
- **No letter-spacing / tracking.** `iced::widget::text` exposes size, line height, font, shaping, alignment — nothing else. Uppercase micro-labels will be tight; compensate with size and colour, not tracking.
- **No per-side borders.** `Border` is one colour/width for all four sides. A "coloured left edge" accent (toast, callout) must be built as a `row![]` with a narrow `container` as the first child.
- **Containers do not clip children to their border radius.** `Container::clip(true)` exists but clips to a *rectangular* viewport intersection, so a rounded card with an image at the top still has square image corners. Either give the card square top corners (`[0.0, 0.0, r, r]`) or accept it.
- **`image` cannot be tinted.** `iced::widget::Image` has `width/height/content_fit/filter_method` and nothing else — the shipped PNG icons render in whatever colour they were authored. See §2.9.
- **No spinner/indeterminate progress widget.** `progress_bar` is determinate only; there is no busy indicator in iced 0.12 and no icon font bundled by us. An indeterminate state must be expressed in text (or skipped).
- **No blur / backdrop-filter.** The modal backdrop can only be a translucent flat colour over the key art.
- **Tooltip styling** goes through `container::StyleSheet` (`ContainerStyle::Tooltip`), so it gets radius/border/shadow like any container.
- **`pick_list` lost `icon_size`** in 0.12 (already noted as a TODO in `style/pick_list.rs`); the dropdown arrow size is not controllable without a custom widget. The dropdown *menu* is styled separately via `overlay::menu::StyleSheet` (`style/menu.rs`) and does support background/selected background/border.

### 2.9 Icons: use SVG, not PNG (one Cargo feature)
`iced_style::svg::Appearance` is `{ color: Option<Color> }` — the SVG widget **can be tinted per style, including a `hovered()` variant**, which is exactly what a state-aware icon system needs. It is behind the `svg` feature of the `iced` crate (`svg = ["iced_widget/svg"]`), which is **not** currently enabled in `client/Cargo.toml`.

Recommendation: add `"svg"` to the iced features, ship icons as monochrome SVGs, and implement `svg::StyleSheet for XindelerUpdaterTheme`. Without it, any icon that must appear on both a gold button (dark icon) and a dark surface (light icon) has to be shipped as two separate PNGs. This is a small, contained PR (task 8 in §10) and everything else in this spec works without it.

There is a second reason: the shipped PNGs are tiny bitmaps — `folder.png` is **10 × 8**, `download.png` 14 × 14, `up_right_arrow.png` 12 × 12, `settings.png` 20 × 20, the rest 24 × 24. `settings_panel.rs` already upscales the 10 × 8 folder to 15 × 15, and on any HiDPI display all of them resample soft. Nothing about the palette fixes blurry icons; only vector sources do.

---

## 3. Colour palette

Replace the contents of the colour block in `client/src/gui/style/mod.rs`. Keep `rgb8()` and add an `rgba8()` sibling; every constant below is given as hex + the exact `rgb8(...)` call.

### 3.1 Surfaces (indigo-black ramp, derived from the key art)

| Constant | Hex | `rgb8(...)` | Use |
|---|---|---|---|
| `INK_900` | `#0B0C12` | `rgb8(11, 12, 18)` | Window background (`application::Appearance.background_color`), deepest wells |
| `INK_800` | `#12141C` | `rgb8(18, 20, 28)` | Opaque panel background (changelog/server-browser body), `ContainerStyle::Dark` |
| `INK_700` | `#191C27` | `rgb8(25, 28, 39)` | Elevated surfaces: modal dialog, toast, news card, tooltip |
| `INK_600` | `#222634` | `rgb8(34, 38, 52)` | Controls: secondary button, text input, pick list, checkbox (unchecked), selected list row |
| `INK_500` | `#2E3346` | `rgb8(46, 51, 70)` | Hover of the above, strong dividers |

Translucent helpers (use `Color::from_rgba`):

| Constant | Value | Use |
|---|---|---|
| `SCRIM` | `rgba(11, 12, 18, 0.78)` | Modal backdrop |
| `PANEL_VEIL` | `rgba(18, 20, 28, 0.88)` | Opaque-ish panels that should let a hint of key art through |
| `WHITE_A06` | `rgba(255,255,255,0.06)` | Hairline borders on dark surfaces |
| `WHITE_A10` | `rgba(255,255,255,0.10)` | Borders on elevated surfaces, ghost-button hover fill |
| `WHITE_A16` | `rgba(255,255,255,0.16)` | Hover border, scrollbar thumb |
| `WHITE_A24` | `rgba(255,255,255,0.24)` | Checkbox border, scrollbar thumb hover |
| `SHADOW_KEY` | `rgba(0, 0, 0, 0.55)` | Modal / elevated shadow colour |

### 3.2 Text

| Constant | Hex | `rgb8(...)` | Contrast on `INK_800` | Use |
|---|---|---|---|---|
| `TEXT_PRIMARY` | `#F2F3F7` | `rgb8(242, 243, 247)` | 16.6:1 | Body, headings, primary labels |
| `TEXT_SECONDARY` | `#A8AEC2` | `rgb8(168, 174, 194)` | 8.3:1 | Supporting copy, list metadata |
| `TEXT_MUTED` | `#868EA3` | `rgb8(134, 142, 163)` | 5.6:1 (4.6:1 on `INK_600`) | Field labels, captions, timestamps |
| `TEXT_ON_ACCENT` | `#0B0C12` | `rgb8(11, 12, 18)` | 8.5:1 *on gold* | Label on a gold button |

### 3.3 Accents

| Constant | Hex | `rgb8(...)` | Role |
|---|---|---|---|
| `GOLD_400` | `#F2B457` | `rgb8(242, 180, 87)` | Primary **hover**, accent text on dark |
| `GOLD_500` | `#E8A33D` | `rgb8(232, 163, 61)` | **Primary accent** — Play/Download button, progress fill, checkbox on, input focus ring |
| `GOLD_600` | `#C9862A` | `rgb8(201, 134, 42)` | Accent border, selected-state fill |
| `GOLD_700` | `#B0731F` | `rgb8(176, 115, 31)` | Primary **pressed** (a deliberate two-step drop, per Fluent 2 — 4.9:1 with the ink label, still AA) |
| `GOLD_GLOW` | `rgba(232, 163, 61, 0.35)` | — | Shadow colour under the primary button on hover |
| `ARCANE_400` | `#6E60E2` | `rgb8(110, 96, 226)` | Secondary-accent hover |
| `ARCANE_500` | `#6355D8` | `rgb8(99, 85, 216)` | Secondary accent — "Update" when it competes with Play, selection highlight |
| `ARCANE_600` | `#5346BC` | `rgb8(83, 70, 188)` | Secondary-accent pressed |
| `CRIMSON_400` | `#CC3E43` | `rgb8(204, 62, 67)` | Danger hover |
| `CRIMSON_500` | `#C0383C` | `rgb8(192, 56, 60)` | Danger fill |
| `CRIMSON_600` | `#A32F33` | `rgb8(163, 47, 51)` | Danger pressed |
| `DANGER_TEXT` | `#F2777A` | `rgb8(242, 119, 122)` | Error copy on dark (6.7:1) |
| `SUCCESS_TEXT` | `#3ECF8E` | `rgb8(62, 207, 142)` | Success copy / toast dot on dark (9.2:1) |

All pairings in this spec were checked for WCAG AA (≥ 4.5:1 for normal text): gold-on-ink label 8.5:1, white-on-arcane 4.96:1, white-on-crimson 4.89:1, gold text on `INK_700` 7.9:1.

### 3.4 Constants to delete

`LIME_GREEN`, `CORNFLOWER_BLUE`, `BLOG_POST_BACKGROUND_BLUE`, `NAVY_BLUE`, `LIGHT_NAVY_BLUE`, `SLATE`, `LILAC`, `BRIGHT_ORANGE`, `TOMATO_RED`, `ALMOST_BLACK`, `ALMOST_BLACK2`, `TRANSPARENT_WHITE`, `DARK_WHITE`, `MEDIUM_GREY`, `LIGHT_GREY`, `VERY_DARK_GREY`.

**Keep** the `lazy_static` brand colours (`DISCORD_BLURPLE`, `MASTODON_PURPLE`, `REDDIT_ORANGE`, `YOUTUBE_RED`, `TWITCH_PURPLE`) — third-party brand marks are not ours to re-tint.

Also change `application::StyleSheet::appearance` to `background_color: INK_900`, `text_color: TEXT_PRIMARY`.

### 3.5 Shape and spacing tokens

Add as consts next to the colours:

```
RADIUS_SM   = 4.0    // progress bar, small icon buttons, chips, status dots
RADIUS_MD   = 6.0    // default: buttons, inputs, pick lists, list rows
RADIUS_LG   = 10.0   // primary CTA, cards, elevated surfaces
RADIUS_XL   = 14.0   // modal dialog
RADIUS_PILL = 999.0  // social/brand chips only
BORDER_HAIRLINE = 1.0
```

These are deliberately tighter than the 8/12/16 web consensus — see the last paragraph of §1: launchers sit at 2–4 px (Steam 2, itch 3, Fluent control default 4), and a voxel game's UI should not look rounder than its art.

Spacing uses a 4 px grid: `4, 8, 12, 16, 20, 24, 32`. Panel side padding is **20** everywhere (today it is 20 in the game panel, 10 in settings, 20 in news — unify).

---

## 4. Typography

Poppins is already embedded in four weights (`POPPINS_BOLD/MEDIUM/FONT/LIGHT`), loaded in `gui/mod.rs`. No new font needed.

| Role | Font | Size | Colour | Notes |
|---|---|---|---|---|
| Primary CTA label | Bold | 26 | `TEXT_ON_ACCENT` | Uppercase (`PLAY`, `DOWNLOAD`, `RETRY`) |
| Secondary CTA label (stacked, 2 lines) | Medium | 15 | `TEXT_PRIMARY` | Sentence case |
| Modal title | Bold | 20 | `TEXT_PRIMARY` | |
| Panel heading (`heading_with_rule`) | Bold | 15 | `TEXT_PRIMARY` | |
| Section/field label | Medium | 10 | `TEXT_MUTED` | UPPERCASE |
| Body | Regular | 14 | `TEXT_PRIMARY` | line height 1.5 |
| Body small / supporting | Regular | 13 | `TEXT_SECONDARY` | line height 1.5 |
| Meta / caption / timestamp | Regular | 11 | `TEXT_MUTED` | |
| Numeric emphasis (%, MB/s) | Bold | 22 / 13 | `TEXT_PRIMARY` | |
| Dialog / modal body | Regular | 14 | `TEXT_SECONDARY` | |
| Button in modal | Medium (secondary) / Bold (primary) | 14 | per §5 | |

Rules:
- **Poppins Light is for display sizes only (≥ 20).** Today it is used at 12 in the changelog body and at 16 for news titles, where it reads thin and fuzzy on dark backgrounds. Switch both to Regular/Medium.
- Uppercase is used in exactly two places: the primary CTA label, and 10 px field/section labels. Nowhere else.
- `Settings.default_text_size` in `gui/mod.rs` is currently **20.0** — lower it to **14.0** so widgets that don't set a size (checkbox labels, tooltip text) inherit something sane.

---

## 5. Button system

### 5.1 New style enum

Replace `ButtonStyle` in `client/src/gui/style/button.rs` with an intent-based set (the old variants are shape-based, which is why every new button needed a new colour):

```rust
pub enum ButtonStyle {
    Primary,            // the one action the current state is about
    Secondary,          // a competing but subordinate action
    Accent,             // coloured-but-not-primary (Update next to Play)
    Ghost,              // low-emphasis text/icon action (nav links, "Not now")
    Danger,             // destructive-ish (Cancel download)
    Icon,               // square icon-only (settings gear, folder, help)
    Chip(BrowserButtonStyle), // social/brand pills — colours unchanged
    ListRow(ServerListEntryButtonState),
    ColumnHeading,
    Transparent,
}
```

`ButtonState::{Enabled, Disabled}` disappears from the style enum: iced already routes to `disabled()` when `on_press` is absent, and the call sites already know whether they set `on_press`. Delete the variant and implement `disabled()` instead (§2.6).

**Every variant implements all four of `active` / `hovered` / `pressed` / `disabled`.**

> Gradient note: a top-to-bottom `GOLD_400 → GOLD_500` gradient on the primary button was considered and rejected — see §2.2, it would cost the glow shadow on wgpu. If a gradient is ever wanted, put it on a *non-shadowed* decorative container (e.g. a thin divider under the logo), never on a button.

### 5.2 The spec, per variant

| | background | label | border | radius | shadow |
|---|---|---|---|---|---|
| **Primary** active | `GOLD_500` | `TEXT_ON_ACCENT` | none | `RADIUS_LG` (10) | `offset (0,2) blur 8` `rgba(0,0,0,0.35)` |
| Primary hover | `GOLD_400` | `TEXT_ON_ACCENT` | none | 10 | `offset (0,4) blur 18` `GOLD_GLOW` |
| Primary pressed | `GOLD_700` | `TEXT_ON_ACCENT` | none | 10 | none |
| Primary disabled | `INK_600` | `TEXT_MUTED` | 1px `WHITE_A06` | 10 | none |
| **Secondary** active | `INK_600` | `TEXT_SECONDARY` | 1px `WHITE_A10` | `RADIUS_MD` (6) | none |
| Secondary hover | `INK_500` | `TEXT_PRIMARY` | 1px `WHITE_A16` | 6 | `offset (0,2) blur 8` `rgba(0,0,0,0.30)` |
| Secondary pressed | `INK_700` | `TEXT_SECONDARY` | 1px `WHITE_A10` | 6 | none |
| Secondary disabled | `INK_700` | `TEXT_MUTED` | 1px `WHITE_A06` | 6 | none |
| **Accent** active | `ARCANE_500` | `TEXT_SECONDARY` | none | 6 | `offset (0,2) blur 8` `rgba(0,0,0,0.35)` |
| Accent hover | `ARCANE_400` | `TEXT_PRIMARY` | none | 6 | `offset (0,3) blur 12` `rgba(0,0,0,0.40)` |
| Accent pressed | `ARCANE_600` | `TEXT_PRIMARY` | none | 6 | none |
| Accent disabled | `INK_600` | `TEXT_MUTED` | 1px `WHITE_A06` | 6 | none |
| **Ghost** active | transparent | `TEXT_SECONDARY` | none | 6 | none |
| Ghost hover | `WHITE_A10` | `TEXT_PRIMARY` | none | 6 | none |
| Ghost pressed | `WHITE_A16` | `TEXT_PRIMARY` | none | 6 | none |
| Ghost disabled | transparent | `TEXT_MUTED` | none | 6 | none |
| **Danger** active | transparent | `DANGER_TEXT` | 1px `rgba(192,56,60,0.55)` | 6 | none |
| Danger hover | `rgba(192,56,60,0.16)` | `DANGER_TEXT` | 1px `CRIMSON_500` | 6 | none |
| Danger pressed | `rgba(192,56,60,0.28)` | `TEXT_PRIMARY` | 1px `CRIMSON_600` | 6 | none |
| Danger disabled | transparent | `TEXT_MUTED` | 1px `WHITE_A06` | 6 | none |
| **Icon** active | transparent | — | none | `RADIUS_SM` (4) | none |
| Icon hover | `WHITE_A10` | — | none | 4 | none |
| Icon pressed | `WHITE_A16` | — | none | 4 | none |
| **Chip** | brand colour (unchanged) | white | none | `RADIUS_PILL` | hover: `+8%` lighten, pressed: `-8%` |
| **ListRow** selected | `INK_600` | `TEXT_PRIMARY` | none | `RADIUS_SM` (4) | none |
| ListRow selected hover | `INK_500` | `TEXT_PRIMARY` | none | 4 | none |
| ListRow not-selected | transparent | `TEXT_SECONDARY` | none | 4 | none |
| ListRow not-selected hover | `WHITE_A06` | `TEXT_PRIMARY` | none | 4 | none |

Four conventions hold the table together, and all four should survive any future button that gets added:
- **Filled buttons** move along their own ramp: `500` active → `400` hover (*lighter* — this is dark mode, §1) → `700`/`600` pressed. Never `color * 1.1`.
- **Pressed is a jump, not a nuance.** `GOLD_500 → GOLD_700` skips a step on purpose; Fluent 2 drops three. A pressed state one shade off the resting state is invisible on a 120 Hz screen with no animation to sell it.
- **Transparent buttons** (Ghost, Icon, ListRow) use a white state layer that *strengthens* on press — `WHITE_A10` hover → `WHITE_A16` pressed (Material 3's verified 8 % → 12 %, nudged up because it sits on a near-black surface). Pressed must never be weaker than hover.
- **Two signals per state, never one.** Filled buttons change fill *and* shadow (shadow appears on hover, vanishes on press — the button reads as pushed in without any animation, §2.5). Secondary/Accent additionally lift their **label** from `TEXT_SECONDARY` to `TEXT_PRIMARY` on hover, which is the trick Steam uses (`#D2EFA9 → #fff`) and costs nothing.

Helper to keep: `color_multiply` is fine for the brand chips; everywhere else use the explicit ramp values above rather than multiplying.

On **disabled**: this spec uses a neutral surface (`INK_600` + `TEXT_MUTED`) rather than a faded accent, because a washed-out gold Play button is what the app does today and it reads as broken. The documented alternative is GOG's — the *same* fill at `0.3` alpha, one token, no extra colours — and it is worth trying on the `CHECKING FOR UPDATES…` state specifically, where "this button is temporarily busy" is closer to the truth than "this button is unavailable". Whichever is chosen, it must be written into `disabled()` explicitly; never let iced's default alpha-halving run on top of an already-dimmed style (§2.6).

### 5.3 Every button in the app today, and what it becomes

| Where | Today | New style | Size / padding | Label |
|---|---|---|---|---|
| `game_panel.rs` "Launch" (`ReadyToPlay`) | lime, radius 4, 75 px tall, Bold 32 | **Primary** | `height 72`, full width portion 3 | `PLAY` (Bold 26, uppercase) + play icon |
| "Play Offline" (`Offline(true)`) | lime | **Primary** | 72 | `PLAY OFFLINE` (Bold 22 — longer string) |
| "Download" (`WaitForConfirm`) | cornflower | **Primary** | 72 | `DOWNLOAD` + download icon |
| "Try Again" (`Offline(false)`) / "Retry" | cornflower | **Primary** | 72 | `RETRY` + refresh icon |
| "Checking…" (`Checking`) | cornflower, disabled | **Primary disabled** | 72 | `CHECKING FOR UPDATES…` (Bold 18) |
| "Playing" (`Playing`) | lime, disabled | **Primary disabled** | 72 | `RUNNING` |
| "Play" in the side-by-side `UpdateAvailable` row | lime, portion 2 | **Primary** | 72, portion 2 | `PLAY` |
| "Update <version>" next to it | cornflower, portion 1 | **Accent** | 72, portion 1 | `Update` Medium 15 + version Regular 11 `TEXT_PRIMARY @ 0.75` |
| "Server Browser" | cornflower, radius 4, portion 1 | **Secondary** | 72, portion 1 | `Server` / `Browser` Medium 15 |
| "Cancel" (during download) | solid tomato, full width, 36 px | **Danger** | `height 36`, full width | `Cancel` Medium 14 |
| Settings gear (`game_panel.rs`) | transparent, radius 10, no bg | **Icon** | `32 × 32`, padding 6 | icon 18 px |
| Update-prompt "Not now" | slate fill | **Ghost** | `height 40`, padding `[0, 18]` | `Not now` Medium 14 |
| Update-prompt "Update" | cornflower | **Primary** | `height 40`, padding `[0, 22]` | `Update now` Bold 14 |
| Launcher-modal "Update now" / "Retry" | cornflower | **Primary** | `height 40`, padding `[0, 22]` | Bold 14 |
| `announcement_panel.rs` "Download XindelerUpdater" | `XindelerUpdaterDownload` (`VERY_DARK_GREY`, radius 25) | **Secondary** | `height 26`, padding `[0, 12]` | Medium 11 + arrow icon |
| `logo_panel.rs` link rows (Manual/Community/Account) | `Transparent`, no hover at all | **Ghost** | padding `[8, 10]`, full width | Medium 13, icon 20 px |
| `settings_panel.rs` folder / book icon buttons | `Transparent`, padding 0 | **Icon** | `24 × 24`, padding 4 | icon 15 px |
| `news_panel.rs` post card | `Transparent`, padding 0 | **Transparent** (keep) | — | card handles its own visuals |
| `changelog_panel.rs` / `server_browser_panel.rs` "…listed here" | `Browser(Extra)` (lime pill) | **Chip**, but recoloured to `INK_600`/`WHITE_A10` | padding `[4, 10]` | Medium 10 |
| Server list rows | `ServerListEntry(…)` navy/dark-grey | **ListRow** | unchanged geometry | |
| Column headings | `ColumnHeading` | **ColumnHeading** (Medium 10 uppercase `TEXT_MUTED`, hover `TEXT_PRIMARY`) | | |
| Social chips (`BrowserButtonStyle`) | brand colours, radius 25 | **Chip** (unchanged colours) | | |

Height note: the primary block goes from 75 → **72** px and the row gap from 10 → **12**, to sit on the 4 px grid.

### 5.4 Checkbox (`style/checkbox.rs`)

`checkbox::Appearance` gives us `background`, `icon_color`, `border`, `text_color` — no shadow (§2.4).

| State | background | border | icon | label |
|---|---|---|---|---|
| unchecked | `INK_600` | 1px `WHITE_A24`, radius `RADIUS_SM` (4) | — | `TEXT_SECONDARY` |
| unchecked hover | `INK_500` | 1px `GOLD_500 @ 0.6` | — | `TEXT_PRIMARY` |
| checked | `GOLD_500` | 1px `GOLD_500` | `TEXT_ON_ACCENT` | `TEXT_PRIMARY` |
| checked hover | `GOLD_400` | 1px `GOLD_400` | `TEXT_ON_ACCENT` | `TEXT_PRIMARY` |
| disabled (both) | `INK_700` | 1px `WHITE_A06` | `TEXT_MUTED` | `TEXT_MUTED` |

Call-site changes in `settings_panel.rs`: `.size(18)`, `.spacing(10)`, `.text_size(13)`.

### 5.5 Focus

iced 0.12 gives buttons no keyboard focus state (there is no `focused()` on `button::StyleSheet`; only `text_input` has focus). Text inputs get a focus ring: border `1px GOLD_500`, radius `RADIUS_MD`. Nothing else can show focus — do not design for it.

---

## 6. Modals and pop-ups

Two dialogs exist: `game_panel.rs::update_prompt_dialog` (game update, dismissible) and `default.rs::launcher_update_dialog` (launcher update, mandatory, three states). They should share one shell.

### 6.1 Shared shell — add `modal_shell()` to `custom_widgets.rs`

```
modal_shell(eyebrow: (Color, &str), title: &str, body: Element, actions: Row) -> Element
```

- **Backdrop** (`ContainerStyle::ModalBackdrop`): `SCRIM` = `rgba(11,12,18,0.78)`, fills the window, centres the card. No blur is possible (§2.8) — the higher opacity compensates.
- **Card** (`ContainerStyle::ModalDialog`): background `INK_700`, border `1px WHITE_A10`, radius `RADIUS_XL` (14), shadow `offset (0, 16)`, `blur_radius 40`, colour `SHADOW_KEY`. Width **420** (today 380), `align_x` left.
- **Card padding**: `[24, 28, 20, 28]` (top, right, bottom, left).
- **Internal layout** (column, `spacing 0`, explicit gaps):
  1. **Eyebrow row** — an 8 × 8 dot (`container` fixed size, radius `4`, background = state colour) + uppercase Medium 10 `TEXT_MUTED` label. Gap 8. *(gold = update available, arcane = in progress, crimson = failed.)*
  2. gap 12
  3. **Title** — Bold 20 `TEXT_PRIMARY`, left aligned.
  4. gap 8
  5. **Body** — Regular 14 `TEXT_SECONDARY`, line height 1.5, left aligned, `width Fill`.
  6. gap 24
  7. **Actions** — `row![].spacing(10)` pushed to the right with a leading `horizontal_space()` (in 0.12 it takes no argument and fills the available width). Secondary/Ghost on the left, Primary rightmost (desktop convention).

Left-aligned text with right-aligned actions is the single biggest "this is a real app" change to the dialogs; the current centred-everything layout is what reads as improvised.

### 6.2 Game update prompt (`update_prompt_dialog`)

- Eyebrow: `GOLD_500` dot + `GAME UPDATE`
- Title: `Version {version} is available`
- Body: `You can install it now, or keep playing on your current version and update later.`
- Actions: **Ghost** `Not now` → `DismissUpdatePrompt`; **Primary** `Update now` → `ConfirmUpdate`.

### 6.3 Launcher update dialog (`launcher_update_dialog`)

Same shell; the three states differ only in eyebrow/title/body/actions:

| State | Eyebrow | Title | Body | Actions |
|---|---|---|---|---|
| `Prompt` | `GOLD_500` · `LAUNCHER UPDATE REQUIRED` | `Update to v{version}` | `The launcher needs to update before you can download or play. It only takes a moment.` | **Primary** `Update now` |
| `Applying` | `ARCANE_500` · `INSTALLING` | `Updating the launcher…` | `Downloading and installing v{version}. The launcher will restart on its own.` | none — plus a full-width 6 px progress bar at 100 % in `INK_600`/`ARCANE_500` as a *static* "in progress" bar (no spinner exists, §2.8) |
| `Failed` | `CRIMSON_500` · `UPDATE FAILED` | `Couldn't update to v{version}` | reason in Regular 14 `TEXT_SECONDARY` + `Check your connection and try again.` | **Primary** `Try again` |

For `Failed`, wrap the raw `reason` string in a callout: `container` with background `rgba(192,56,60,0.12)`, border `1px rgba(192,56,60,0.35)`, radius `RADIUS_MD`, padding `[10, 12]`, text Regular 12 `DANGER_TEXT`. Raw error text should never be a bare paragraph.

### 6.4 Tooltips

`ContainerStyle::Tooltip`: background `INK_700`, border `1px WHITE_A10`, radius `RADIUS_MD`, shadow `offset (0,4) blur 12 rgba(0,0,0,0.45)`, padding `[6, 10]`, text 12 `TEXT_SECONDARY`. Today it is navy with a grey 1 px border and no radius or padding.

---

## 7. Messages, states and toasts

### 7.1 Toast (`default.rs::toast_banner`, `ContainerStyle::Toast`)

Today: a solid lime bar with 12 px white text. New: a neutral surface with a status dot — the pattern Linear/Notion/GitHub Desktop all use, and the only way to get a "coloured accent edge" given that `Border` is uniform (§2.8).

- Container: background `INK_700`, border `1px WHITE_A10`, radius `RADIUS_MD`, shadow `offset (0,4) blur 16 rgba(0,0,0,0.45)`, padding `[10, 14]`.
- Content: `row![].spacing(10).align_items(Center)` →
  1. 8 × 8 dot, radius 4, background `SUCCESS_TEXT`;
  2. message, Regular 13 `TEXT_PRIMARY`, left aligned (not centred).
- Outer padding in the sidebar column stays 10; width `Fill`.
- Duration stays 5 s. It cannot fade (no animation, §2.5) — it will pop out, which is fine.

Optional follow-up: a `ToastKind { Success, Error }` so failures can reuse the same shell with a `DANGER_TEXT` dot.

### 7.2 Download progress area (`game_panel.rs`, `InProgress`)

Today: step label 14, a row of 12 px strings, a **28 px tall** progress bar, then a full-width red Cancel. The fat bar and the flat row of same-size numbers are the main offenders.

New layout (column, `spacing 0`, explicit gaps, inside the existing `[10, 20, 20, 20]` padding):

1. **Row** (`align_items Center`): step caption on the left — Medium 10 uppercase `TEXT_MUTED`, one of `DOWNLOADING` / `UNZIPPING` / `DELETING` / `FINALIZING`; `horizontal_space()`; percentage on the right — **Bold 22 `TEXT_PRIMARY`**, `format!("{percent:.0}%")`.
2. gap 8
3. **Progress bar** — `height 6`, `border_radius RADIUS_SM`, track `INK_600`, bar `GOLD_500`. (No border/shadow available, §2.4.)
4. gap 10
5. **Meta row** (`spacing 6`, `align_items Center`): `1.2 GB / 3.4 GB` Regular 12 `TEXT_SECONDARY` · dot separator `·` `TEXT_MUTED` · `8.4 MB/s` **Medium 12 `TEXT_PRIMARY`** · `horizontal_space()` · `02:31 left` Regular 12 `TEXT_SECONDARY`.
   The `DOWNLOAD_ICON` image is dropped from this row — the caption already says what's happening, and the untintable PNG (§2.8) fights the new palette.
6. gap 16
7. **Cancel** — **Danger** style, `height 36`, full width.

### 7.3 Status strings

Copy changes, all in `game_panel.rs`:

| Current | New |
|---|---|
| `Checking...` | `CHECKING FOR UPDATES…` (real ellipsis) |
| `Launch` | `PLAY` |
| `Playing` | `RUNNING` |
| `Play Offline` | `PLAY OFFLINE` |
| `Try Again` / `Retry` | `RETRY` |
| `Unknown` step | `PREPARING` |
| `Pre-Alpha (0.26.0) — 0.26.1 available` | keep the string, but render as: `Pre-Alpha · v0.26.0` Regular 11 `TEXT_MUTED`, and when an update exists append a separate `text` in `GOLD_400` reading `v0.26.1 available` |

### 7.4 Error / offline states

- `GamePanelState::Retry` and `Offline(false)` should show a one-line explanation *above* the button, Regular 12 `DANGER_TEXT` (`Couldn't reach the update server.` / `Download failed.`), not just a relabelled button. Add a `TextStyle::Danger` and `TextStyle::Success` to `style/text.rs` (and delete `BrightOrange` / `TomatoRed` / `Lilac`).

---

## 8. Settings panel

`settings_panel.rs` today is five `container(row![...])` blocks of pick lists with 10 px uppercase labels, padded `[40, 10]`, with no grouping and inconsistent widths. Structure it, without changing a single message or field.

### 8.1 Structure

```
Settings                      (heading_with_rule, unchanged widget)
  GRAPHICS                    section label — Medium 10 uppercase TEXT_MUTED
    [Graphics device      ]   full width
    [Graphics mode ] [Log level ]      two columns, spacing 12
  GAME
    [Server        ] [Channel   ]
    [Assets override .......] [📁]
  ADVANCED
    [Environment variables ..............]
  ┌ card ─────────────────────────────────┐
  │ ☑ Auto-update game                    │
  │ ☑ Auto-update launcher                │
  │ When on, a new version downloads …    │
  └───────────────────────────────────────┘
```

- Outer padding `[16, 20]` (was `[40, 10]`), section gap **20**, field gap **12**, label→control gap **6**.
- Section labels are plain text, not `heading_with_rule` (that widget stays reserved for panel titles).
- Field labels: Medium 10 uppercase `TEXT_MUTED`, no left padding hack (`[0,0,0,3]` goes away).
- All controls `height 32` (was 30), `text_size 13` (was 12), `padding [0, 10]` with fixed height.
- The auto-update block becomes a card: `container` background `INK_700`, border `1px WHITE_A06`, radius `RADIUS_LG`, padding `[12, 14]`, `spacing 8`; helper text Regular 11 `TEXT_MUTED`, line height 1.5.
- Keep every tooltip; restyle comes free via §6.4.

### 8.2 Control styling

**`pick_list` (`style/pick_list.rs`)** — implement `hovered()` properly (today it returns `active()`):

| State | background | border | text | handle |
|---|---|---|---|---|
| active | `INK_600` | 1px `WHITE_A10`, radius `RADIUS_MD` | `TEXT_PRIMARY` | `TEXT_SECONDARY` |
| hovered | `INK_500` | 1px `WHITE_A16` | `TEXT_PRIMARY` | `TEXT_PRIMARY` |

placeholder `TEXT_MUTED`. (Handle *size* is not controllable in 0.12, §2.8.)

**`overlay::menu` (`style/menu.rs`)**: background `INK_700`, selected background `INK_600`, text `TEXT_PRIMARY`, selected text `TEXT_PRIMARY`, border `1px WHITE_A10` radius `RADIUS_MD`.

**`text_input` (`style/text_input.rs`)**: active background `INK_600`, border `1px WHITE_A10` radius `RADIUS_MD`; **focused** border `1px GOLD_500` (this is the app's only real focus affordance, §5.5); value `TEXT_PRIMARY`, placeholder `TEXT_MUTED`, selection `GOLD_500 @ 0.35`; disabled background `INK_700`, text `TEXT_MUTED`.

**`checkbox`**: §5.4.

---

## 9. The rest of the UI

Not a redesign of each panel — just the consistency fixes that the palette change forces, plus the two or three cheap wins per panel.

**`custom_widgets.rs::heading_with_rule`** — the rule line is 1 px but pure `Color::WHITE` (`horizontal_rule(8)` is the 8 px layout slot; `RuleStyle.width` is the line thickness), which makes it the loudest unintentional element on screen. Change `RuleStyle` to `color: WHITE_A10`, `width: 1`, `radius: 0`; heading text Bold **15**; leading stub 13 px wide stays.

**`style/scrollable.rs`** — the scroller is currently `ALMOST_BLACK` (black on a black panel = invisible). New: track transparent; scroller `WHITE_A16`, radius 4, width 6; hovered scroller `WHITE_A24`, track `WHITE_A06`.

**`style/progress_bar.rs`** — track `INK_600`, bar `GOLD_500`, `border_radius RADIUS_SM`.

**`logo_panel.rs`** — link rows become **Ghost** buttons (they currently have zero hover feedback, which makes them look non-clickable), text Medium 13 `TEXT_SECONDARY` → `TEXT_PRIMARY` on hover, icons 20 px, row padding `[8, 10]`, column `spacing 2`, block top padding 32 (was 40).

**`news_panel.rs`** — `ContainerStyle::BlogPost` goes from `#3D548F` blue to `INK_700` with border `1px WHITE_A06` and radius `[0.0, 0.0, RADIUS_LG, RADIUS_LG]` (square top corners because the image above cannot be clipped, §2.8). Category label `Development`: `TextStyle::Lilac` → `GOLD_400`, Medium 10 uppercase. Post title: Light 16 → **Medium 15** `TEXT_PRIMARY`. Description: 11 → 12 `TEXT_SECONDARY`. `LoadingBlogPost` placeholder: background `INK_800`, border `1px WHITE_A06`, text `TEXT_MUTED`.

**`changelog_panel.rs`** — `ContainerStyle::ChangelogHeader` black → `INK_800`; `ContainerStyle::Dark` → `PANEL_VEIL` (`rgba(18,20,28,0.88)`) so a hint of key art shows through the long body panel, which is a cheap and very effective depth cue. Version heading Bold 20 `TEXT_PRIMARY`; section headings Medium 13 `TEXT_SECONDARY`; bullets Light 12 → **Regular 13** `TEXT_SECONDARY` with line height 1.5; the lime `Browser(Extra)` chip becomes the neutral **Chip**.

**`announcement_panel.rs`** — the solid `BRIGHT_ORANGE` bar clashes head-on with the new gold accent. New `ContainerStyle::Announcement`: background `rgba(232,163,61,0.14)`, border `1px rgba(232,163,61,0.30)`, radius `RADIUS_MD`, height 44 (was 50), padding `[0, 16]`; text Medium 13 `GOLD_400` (not `TextStyle::Dark`); the "Download XindelerUpdater" button becomes **Secondary**, height 26.

**`server_browser_panel.rs`** — rows via **ListRow** (§5.2); `ContainerStyle::ColumnHeading` → background transparent with a 1 px `WHITE_A06` bottom edge faked by a `horizontal_rule` under the row, headings Medium 10 uppercase `TEXT_MUTED`; `ContainerStyle::ExtraBrowser` (lime pill) → `INK_600` + `WHITE_A10` border, radius `RADIUS_PILL`, text Medium 11 `TEXT_SECONDARY`; detail pane body 14 → 13; the `TomatoRed` error text → `DANGER_TEXT`. Ping icons stay as-is (PNGs, untintable).

**`gui/mod.rs`** — `default_text_size: 14.0`; window `min_size` 400×250 is too small for the 360 px sidebar + content, raise to **880 × 560**.

**Unchanged on purpose**: social/brand chip colours, the background-image rotation, all layout proportions (360 px sidebar, 248 px news column), every message type and state machine.

---

## 10. Task list (priority order, small PRs)

Ordered by visual impact per unit of effort. Each item is independently shippable and leaves the app in a working state.

**1 — Palette and token foundation** (`style/mod.rs`, `style/text.rs`, `gui/mod.rs`)
Add the `INK_*` / `GOLD_*` (400/500/600/700) / `ARCANE_*` / `CRIMSON_*` / text / alpha / radius constants; delete the old colour consts; app background → `INK_900`; **`TextStyle::Normal` → `color: None`** (§2.7 — prerequisite for everything else); add `TextStyle::{Secondary, Muted, Accent, Danger, Success}`; `default_text_size` 14. Mechanical compile-error-driven sweep across all style modules.
*Impact: high. Effort: medium (touches every style file, but each change is a constant swap).*

**2 — Button system rewrite** (`style/button.rs` + call sites)
New intent-based `ButtonStyle`; all four state methods on every variant; delete `shadow_offset` usage (§2.4) and `ButtonState`; map the call sites per §5.3.
*Impact: highest single change. Effort: medium.*

**3 — Game panel: hierarchy, sizes, copy** (`components/game_panel.rs`)
Primary/Secondary/Accent assignment, 72 px heights, uppercase CTA labels at 26, 12 px gaps, version line restyle, status-string copy (§7.3), error line above the button (§7.4).
*Impact: highest — this is the screen everyone looks at. Effort: low-medium.*

**4 — Download progress redesign** (`components/game_panel.rs`, `style/progress_bar.rs`)
6 px bar, big percentage, caption/meta typography, Danger Cancel (§7.2).
*Impact: high. Effort: low.*

**5 — Modal shell** (`custom_widgets.rs`, `style/container.rs`, `components/game_panel.rs`, `views/default.rs`)
`modal_shell()`, new `ModalBackdrop` / `ModalDialog` appearances, eyebrow + left-aligned text + right-aligned actions, the three launcher states, the error callout (§6).
*Impact: high. Effort: medium.*

**6 — Toast + status text** (`views/default.rs`, `style/container.rs`)
Neutral toast surface with a status dot; `TextStyle::{Danger, Success}` in use (§7.1).
*Impact: medium. Effort: low.*

**7 — Settings panel restructure** (`components/settings_panel.rs`, `style/{pick_list,menu,text_input,checkbox}.rs`)
Sections, consistent 32 px controls, the auto-update card, real `hovered()`/focus styles (§8).
*Impact: medium-high. Effort: medium.*

**8 — Icon system via SVG** (`Cargo.toml`, `assets.rs`, new `style/svg.rs`)
Enable the iced `svg` feature, implement `svg::StyleSheet`, replace the button-adjacent PNGs (play, download, refresh, settings, folder, book, arrow) with tintable SVGs so an icon can be dark on the gold button and light on dark surfaces (§2.9). Adds the play/refresh icons that tasks 3 needs — until this lands, ship those buttons text-only.
*Impact: medium. Effort: medium (needs new asset files).*

**9 — Surrounding panels** (`logo_panel`, `news_panel`, `changelog_panel`, `announcement_panel`, `server_browser_panel`, `style/{rule,scrollable}.rs`)
Rules at `WHITE_A10`, visible scrollbars, panel-by-panel colour/typography fixes per §9.
*Impact: medium (removes the remaining colour clashes). Effort: medium, easily split per panel.*

**10 — Polish pass**
`min_size` bump, spacing audit against the 4 px grid, a `--mock-state` walkthrough of all eight game-panel states (`ready`, `checking`, `wait-for-confirm`, `update-prompt`, `update-available`, `offline-playable`, `offline-unplayable`, `retry`) plus both launcher-modal states and the toast, contrast re-check.
Note: `--mock-state` has **no case for the in-progress download**, which is exactly the area task 4 rewrites. Add a `Downloading` variant to `MockGameState` (a `GamePanelState::Updating { btnstate: InProgress }` plus a canned `Progress::Incomplete`) as part of task 4, otherwise the new progress layout can only be checked against a real multi-GB download.
*Impact: low individually, high in aggregate. Effort: low.*

**Explicitly out of scope** (documented so they don't get re-litigated): state transitions/animations (§2.5), gradient buttons (§2.2), backdrop blur (§2.8), a busy spinner (§2.8), keyboard-focus rings on buttons (§5.5), any change to the `GamePanelState` / `LauncherUpdateState` machines.
