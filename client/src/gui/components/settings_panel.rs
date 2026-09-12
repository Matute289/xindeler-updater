use std::path::PathBuf;

use crate::{
    Result,
    assets::{BOOK_ICON, FOLDER_ICON},
    channels::{Channel, Channels},
    gui::{
        components::GamePanelMessage,
        custom_widgets::heading_with_rule,
        style::{
            button::ButtonStyle, checkbox::CheckboxStyle, container::ContainerStyle,
            text::TextStyle,
        },
        views::{
            Action,
            default::{DefaultViewMessage, Interaction},
        },
        widget::*,
    },
    profiles,
    profiles::Profile,
};
use iced::{
    Alignment, Command, Length,
    alignment::Horizontal,
    widget::{
        Image, button, checkbox, column, container, image, image::Handle, pick_list, row,
        text, text_input, tooltip, tooltip::Position,
    },
};
use rust_i18n::t;
use tracing::debug;

#[derive(Clone, Debug)]
pub enum SettingsPanelMessage {
    WgpuGraphicsDeviceChanged(profiles::WgpuDevice),
    LogLevelChanged(profiles::LogLevel),
    ServerChanged(profiles::Server),
    ChannelChanged(Channel),
    WgpuBackendChanged(profiles::WgpuBackend),
    EnvVarsChanged(String),
    AssetsOverrideChanged(String),
    AssetsOverridePick,
    AssetsOverridePicked(Option<PathBuf>),
    OpenLogsPressed,
    ChannelsLoaded(Result<Channels>),
    AutoUpdateGameToggled(bool),
    AutoUpdateLauncherToggled(bool),
    LanguageChanged(profiles::Language),
}

#[derive(Clone, Debug, Default)]
pub struct SettingsPanelComponent {
    channels: Channels,
}

impl SettingsPanelComponent {
    pub fn update(
        &mut self,
        msg: SettingsPanelMessage,
        active_profile: &Profile,
    ) -> Option<Command<DefaultViewMessage>> {
        match msg {
            SettingsPanelMessage::WgpuGraphicsDeviceChanged(new_device) => {
                let mut profile = active_profile.clone();
                profile.wgpu_device = new_device;
                Some(Command::perform(
                    async { Action::UpdateProfile(profile) },
                    DefaultViewMessage::Action,
                ))
            },
            SettingsPanelMessage::ServerChanged(new_server) => {
                tracing::debug!("new server selected {}", new_server);
                let mut profile = active_profile.clone();
                profile.server = new_server;
                let profile2 = profile.clone();
                Some(Command::batch(vec![
                    Command::perform(
                        async { Action::UpdateProfile(profile2) },
                        DefaultViewMessage::Action,
                    ),
                    Command::perform(async {}, |_| {
                        DefaultViewMessage::GamePanel(GamePanelMessage::StartUpdate)
                    }),
                ]))
            },
            SettingsPanelMessage::ChannelChanged(new_channel) => {
                tracing::debug!("new channel selected {}", new_channel);
                let mut profile = active_profile.clone();
                profile.channel = new_channel;
                let profile2 = profile.clone();
                Some(Command::batch(vec![
                    Command::perform(
                        async { Action::UpdateProfile(profile2) },
                        DefaultViewMessage::Action,
                    ),
                    Command::perform(async {}, |_| {
                        DefaultViewMessage::GamePanel(GamePanelMessage::StartUpdate)
                    }),
                ]))
            },
            SettingsPanelMessage::WgpuBackendChanged(wgpu_backend) => {
                let mut profile = active_profile.clone();
                profile.wgpu_backend = wgpu_backend;
                Some(Command::perform(
                    async { Action::UpdateProfile(profile) },
                    DefaultViewMessage::Action,
                ))
            },
            SettingsPanelMessage::LogLevelChanged(log_level) => {
                let mut profile = active_profile.clone();
                profile.log_level = log_level;
                Some(Command::perform(
                    async { Action::UpdateProfile(profile) },
                    DefaultViewMessage::Action,
                ))
            },
            SettingsPanelMessage::OpenLogsPressed => {
                if let Err(e) = opener::open(active_profile.voxygen_logs_path()) {
                    tracing::error!("Failed to open logs dir: {:?}", e);
                }
                None
            },
            SettingsPanelMessage::EnvVarsChanged(vars) => {
                let mut profile = active_profile.clone();
                profile.env_vars = vars;
                Some(Command::perform(
                    async { Action::UpdateProfile(profile) },
                    DefaultViewMessage::Action,
                ))
            },
            SettingsPanelMessage::AssetsOverrideChanged(assets) => {
                let mut profile = active_profile.clone();
                profile.assets_override = (!assets.is_empty()).then_some(assets);
                Some(Command::perform(
                    async { Action::UpdateProfile(profile) },
                    DefaultViewMessage::Action,
                ))
            },
            SettingsPanelMessage::AssetsOverridePick => Some(Command::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .set_title(t!("settings.assets_override_dialog_title"))
                        .pick_folder()
                        .await
                        .map(|folder| folder.path().to_path_buf())
                },
                |path| {
                    DefaultViewMessage::SettingsPanel(
                        SettingsPanelMessage::AssetsOverridePicked(path),
                    )
                },
            )),
            SettingsPanelMessage::AssetsOverridePicked(Some(path)) => {
                let mut profile = active_profile.clone();
                // TODO: we probably should get rid of to_string_lossy()
                // and display an error on non-utf8 strings here
                let assets = path.to_string_lossy().into_owned();
                let assets = assets.trim().to_owned();
                profile.assets_override = (!assets.is_empty()).then_some(assets);
                Some(Command::perform(
                    async { Action::UpdateProfile(profile) },
                    DefaultViewMessage::Action,
                ))
            },
            SettingsPanelMessage::AssetsOverridePicked(None) => {
                // user closed the menu, chill out
                None
            },
            SettingsPanelMessage::AutoUpdateGameToggled(enabled) => {
                let mut profile = active_profile.clone();
                profile.auto_update_game = enabled;
                Some(Command::perform(
                    async { Action::UpdateProfile(profile) },
                    DefaultViewMessage::Action,
                ))
            },
            SettingsPanelMessage::AutoUpdateLauncherToggled(enabled) => {
                let mut profile = active_profile.clone();
                profile.auto_update_launcher = enabled;
                Some(Command::perform(
                    async { Action::UpdateProfile(profile) },
                    DefaultViewMessage::Action,
                ))
            },
            SettingsPanelMessage::LanguageChanged(new_language) => {
                let mut profile = active_profile.clone();
                profile.language = new_language;
                // Applied immediately rather than waiting for the profile round-trip,
                // so the re-render this message triggers is already in the new
                // language - switching languages must not need a restart.
                rust_i18n::set_locale(new_language.code());
                Some(Command::perform(
                    async { Action::UpdateProfile(profile) },
                    DefaultViewMessage::Action,
                ))
            },
            SettingsPanelMessage::ChannelsLoaded(result) => {
                if let Ok(channels) = result {
                    debug!(?channels, "Fetched available channels:");
                    self.channels = channels;
                }

                None
            },
        }
    }

    pub fn view<'a>(
        &self,
        active_profile: &'a Profile,
    ) -> Element<'a, DefaultViewMessage> {
        const PICK_LIST_PADDING: u16 = 7;
        const FONT_SIZE: u16 = 13;

        let language_picker = column![]
            .spacing(5)
            .push(container(
                text(t!("settings.language_label"))
                    .size(10)
                    .style(TextStyle::Muted),
            ))
            .push(
                tooltip(
                    container(
                        pick_list(
                            profiles::LANGUAGES,
                            Some(active_profile.language),
                            |x| {
                                DefaultViewMessage::SettingsPanel(
                                    SettingsPanelMessage::LanguageChanged(x),
                                )
                            },
                        )
                        .text_size(FONT_SIZE)
                        .padding(PICK_LIST_PADDING)
                        .width(Length::Fill),
                    )
                    .height(Length::Fixed(32.0)),
                    text(t!("settings.language_tooltip")).size(14),
                    Position::Bottom,
                )
                .style(ContainerStyle::Tooltip)
                .gap(5),
            )
            .width(Length::FillPortion(1));

        let graphics_device = column![]
            .spacing(5)
            .push(container(
                text(t!("settings.graphics_device_label"))
                    .size(10)
                    .style(TextStyle::Muted),
            ))
            .push(
                tooltip(
                    container(
                        pick_list(
                            active_profile.supported_wgpu_devices.as_slice(),
                            Some(active_profile.wgpu_device.clone()),
                            |x| {
                                DefaultViewMessage::SettingsPanel(
                                    SettingsPanelMessage::WgpuGraphicsDeviceChanged(x),
                                )
                            },
                        )
                        .text_size(FONT_SIZE)
                        .padding(PICK_LIST_PADDING)
                        .width(Length::Fill),
                    )
                    .height(Length::Fixed(32.0)),
                    text(t!("settings.graphics_device_tooltip")).size(14),
                    Position::Bottom,
                )
                .style(ContainerStyle::Tooltip)
                .gap(5),
            )
            .width(Length::FillPortion(1));

        let graphics_mode = column![]
            .spacing(5)
            .push(container(
                text(t!("settings.graphics_mode_label"))
                    .size(10)
                    .style(TextStyle::Muted),
            ))
            .push(
                tooltip(
                    container(
                        pick_list(
                            active_profile.supported_wgpu_backends.as_slice(),
                            Some(active_profile.wgpu_backend),
                            |x| {
                                DefaultViewMessage::SettingsPanel(
                                    SettingsPanelMessage::WgpuBackendChanged(x),
                                )
                            },
                        )
                        .text_size(FONT_SIZE)
                        .padding(PICK_LIST_PADDING)
                        .width(Length::Fill),
                    )
                    .height(Length::Fixed(32.0)),
                    text(t!("settings.graphics_mode_tooltip")).size(14),
                    Position::Bottom,
                )
                .style(ContainerStyle::Tooltip)
                .gap(5),
            )
            .width(Length::FillPortion(1));

        let log_level = column![]
            .spacing(5)
            .push(
                row![]
                    .spacing(5)
                    .push(container(
                        text(t!("settings.log_level_label"))
                            .size(10)
                            .style(TextStyle::Muted),
                    ))
                    .push(
                        container(
                            button(
                                image(Handle::from_memory(FOLDER_ICON.to_vec()))
                                    .height(Length::Fixed(15.0))
                                    .width(Length::Fixed(15.0)),
                            )
                            .on_press(DefaultViewMessage::SettingsPanel(
                                SettingsPanelMessage::OpenLogsPressed,
                            ))
                            .padding(0)
                            .style(ButtonStyle::Transparent),
                        )
                        .align_x(Horizontal::Right),
                    )
                    .align_items(Alignment::Center),
            )
            .push(
                tooltip(
                    container(
                        pick_list(
                            profiles::LOG_LEVELS,
                            Some(active_profile.log_level),
                            |x| {
                                DefaultViewMessage::SettingsPanel(
                                    SettingsPanelMessage::LogLevelChanged(x),
                                )
                            },
                        )
                        .text_size(FONT_SIZE)
                        .padding(PICK_LIST_PADDING)
                        .width(Length::Fill),
                    )
                    .height(Length::Fixed(32.0)),
                    text(t!("settings.log_level_tooltip")).size(14),
                    Position::Bottom,
                )
                .style(ContainerStyle::Tooltip)
                .gap(5),
            )
            .width(Length::FillPortion(1));

        let server_picker = column![]
            .spacing(5)
            .push(container(
                text(t!("settings.server_label"))
                    .size(10)
                    .style(TextStyle::Muted),
            ))
            .push(
                tooltip(
                    container(
                        pick_list(profiles::SERVERS, Some(active_profile.server), |x| {
                            DefaultViewMessage::SettingsPanel(
                                SettingsPanelMessage::ServerChanged(x),
                            )
                        })
                        .text_size(FONT_SIZE)
                        .padding(PICK_LIST_PADDING)
                        .width(Length::Fill),
                    )
                    .height(Length::Fixed(32.0)),
                    text(t!("settings.server_tooltip")).size(14),
                    Position::Bottom,
                )
                .style(ContainerStyle::Tooltip)
                .gap(5),
            )
            .width(Length::FillPortion(1));

        let help_link =
            "https://book.xindeler.com/players/env-vars.html#veloren_assets_override"
                .to_owned();
        let assets_override = column![]
            .spacing(5)
            .push(
                row![]
                    .spacing(5)
                    .push(container(
                        text(t!("settings.assets_override_label"))
                            .size(10)
                            .style(TextStyle::Muted),
                    ))
                    .push(help_link_button(help_link)),
            )
            .push(
                tooltip(
                    container(
                        row![
                            text_input(
                                &t!("settings.assets_override_placeholder"),
                                active_profile
                                    .assets_override
                                    .as_deref()
                                    .unwrap_or_default(),
                            )
                            .on_input(|path| {
                                DefaultViewMessage::SettingsPanel(
                                    SettingsPanelMessage::AssetsOverrideChanged(path),
                                )
                            })
                            .padding(PICK_LIST_PADDING)
                            .size(FONT_SIZE),
                            button(
                                image(Handle::from_memory(FOLDER_ICON.to_owned()))
                                    .height(Length::Fixed(15.0))
                                    .width(Length::Fixed(15.0))
                            )
                            .on_press(DefaultViewMessage::SettingsPanel(
                                SettingsPanelMessage::AssetsOverridePick,
                            ))
                            .padding(PICK_LIST_PADDING)
                            .style(ButtonStyle::Transparent),
                        ]
                        .spacing(5)
                        .align_items(Alignment::Center),
                    )
                    .height(Length::Fixed(32.0)),
                    text(t!("settings.assets_override_tooltip")).size(14),
                    Position::Bottom,
                )
                .style(
                    // TODO: this and env_vars should probably scream at you for putting
                    // invalid data in
                    ContainerStyle::Tooltip,
                )
                .gap(5),
            )
            .width(Length::Fill);

        let help_link = "https://book.xindeler.com/players/env-vars.html".to_owned();
        let env_vars = column![]
            .spacing(5)
            .push(
                row![]
                    .spacing(5)
                    .push(container(
                        text(t!("settings.env_vars_label"))
                            .size(10)
                            .style(TextStyle::Muted),
                    ))
                    .push(help_link_button(help_link)),
            )
            .push(
                tooltip(
                    container(
                        text_input("FOO=foo, BAR=bar", &active_profile.env_vars)
                            .on_input(|vars| {
                                DefaultViewMessage::SettingsPanel(
                                    SettingsPanelMessage::EnvVarsChanged(vars),
                                )
                            })
                            .padding(PICK_LIST_PADDING)
                            .size(FONT_SIZE),
                    )
                    .height(Length::Fixed(32.0)),
                    text(t!("settings.env_vars_tooltip")).size(14),
                    Position::Bottom,
                )
                .style(ContainerStyle::Tooltip)
                .gap(5),
            )
            .width(Length::FillPortion(2));

        let channel_picker = column![]
            .spacing(5)
            .push(container(
                text(t!("settings.channel_label"))
                    .size(10)
                    .style(TextStyle::Muted),
            ))
            .push(
                tooltip(
                    container(
                        pick_list(
                            self.channels.names.clone(),
                            Some(active_profile.channel.clone()),
                            |x| {
                                DefaultViewMessage::SettingsPanel(
                                    SettingsPanelMessage::ChannelChanged(x),
                                )
                            },
                        )
                        .width(Length::Fill)
                        .text_size(FONT_SIZE)
                        .padding(PICK_LIST_PADDING),
                    )
                    .height(Length::Fixed(32.0)),
                    text(t!("settings.channel_tooltip")).size(14),
                    Position::Bottom,
                )
                .style(ContainerStyle::Tooltip)
                .gap(5),
            )
            .width(Length::FillPortion(1));

        let game_version = active_profile
            .version
            .clone()
            .unwrap_or_else(|| t!("settings.version_not_installed").into_owned());
        let launcher_version = format!("v{}", env!("CARGO_PKG_VERSION"));

        let versions_row = row![]
            .spacing(12)
            .push(
                column![]
                    .spacing(5)
                    .push(container(
                        text(t!("settings.game_version_label"))
                            .size(10)
                            .style(TextStyle::Muted),
                    ))
                    .push(text(game_version).size(FONT_SIZE))
                    .width(Length::FillPortion(1)),
            )
            .push(
                column![]
                    .spacing(5)
                    .push(container(
                        text(t!("settings.launcher_version_label"))
                            .size(10)
                            .style(TextStyle::Muted),
                    ))
                    .push(text(launcher_version).size(FONT_SIZE))
                    .width(Length::FillPortion(1)),
            );

        let general_section = column![]
            .spacing(12)
            .push(section_label(&t!("settings.section_general")))
            .push(row![].push(language_picker))
            .push(versions_row);

        let mut graphics_section = column![]
            .spacing(12)
            .push(section_label(&t!("settings.section_graphics")));
        // Device/backend detection needs to ask the installed game binary what it
        // supports - before that, these two only offer "Auto" (see
        // docs/design/gpu-detection-feasibility.md). Only worth explaining while
        // it's actually true.
        if !active_profile.installed() {
            graphics_section = graphics_section.push(
                text(t!("settings.graphics_auto_only_note"))
                    .size(11)
                    .style(TextStyle::Muted),
            );
        }
        graphics_section = graphics_section.push(row![].push(graphics_device)).push(
            row![]
                .spacing(12)
                .align_items(Alignment::End)
                .push(graphics_mode)
                .push(log_level),
        );

        let game_section = column![]
            .spacing(12)
            .push(section_label(&t!("settings.section_game")))
            .push(
                row![]
                    .spacing(12)
                    .align_items(Alignment::End)
                    .push(server_picker)
                    .push(channel_picker),
            )
            .push(row![].align_items(Alignment::End).push(assets_override));

        let advanced_section = column![]
            .spacing(12)
            .push(section_label(&t!("settings.section_advanced")))
            .push(row![].push(env_vars));

        let auto_update_card = container(
            column![]
                .spacing(8)
                .push(
                    checkbox(
                        t!("settings.auto_update_game_label"),
                        active_profile.auto_update_game,
                    )
                    .style(CheckboxStyle::Default)
                    .size(18)
                    .spacing(10)
                    .text_size(FONT_SIZE)
                    .on_toggle(|enabled| {
                        DefaultViewMessage::SettingsPanel(
                            SettingsPanelMessage::AutoUpdateGameToggled(enabled),
                        )
                    }),
                )
                .push(
                    checkbox(
                        t!("settings.auto_update_launcher_label"),
                        active_profile.auto_update_launcher,
                    )
                    .style(CheckboxStyle::Default)
                    .size(18)
                    .spacing(10)
                    .text_size(FONT_SIZE)
                    .on_toggle(|enabled| {
                        DefaultViewMessage::SettingsPanel(
                            SettingsPanelMessage::AutoUpdateLauncherToggled(enabled),
                        )
                    }),
                )
                .push(
                    text(t!("settings.auto_update_note"))
                        .size(11)
                        .style(TextStyle::Muted),
                ),
        )
        .style(ContainerStyle::Card)
        .padding([12, 14])
        .width(Length::Fill);

        let col = column![]
            .spacing(20)
            .push(general_section)
            .push(graphics_section)
            .push(game_section)
            .push(advanced_section)
            .push(auto_update_card);

        column![]
            .push(heading_with_rule(t!("settings.heading")))
            .push(container(col).padding([16, 20]).height(Length::Shrink))
            .into()
    }
}

fn section_label<'a>(label: &str) -> Element<'a, DefaultViewMessage> {
    text(label.to_uppercase())
        .size(10)
        .style(TextStyle::Muted)
        .into()
}

fn help_link_button(url: String) -> Element<'static, DefaultViewMessage> {
    button(
        Image::new(Handle::from_memory(BOOK_ICON.to_vec()))
            .height(Length::Fixed(15.0))
            .width(Length::Fixed(15.0)),
    )
    .on_press(DefaultViewMessage::Interaction(Interaction::OpenURL(url)))
    .padding(0)
    .style(ButtonStyle::Transparent)
    .into()
}
