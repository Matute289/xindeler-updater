use crate::{
    assets::BACKGROUND_IMAGES,
    channels::Channels,
    gui::{
        background_image::{centered_cover, layered},
        components::{
            AnnouncementPanelComponent, AnnouncementPanelMessage,
            ChangelogPanelComponent, ChangelogPanelMessage, GamePanelComponent,
            GamePanelMessage, LogoPanelComponent, NewsPanelComponent, NewsPanelMessage,
            SERVER_BROWSER_PING_REFRESH, ServerBrowserPanelComponent,
            ServerBrowserPanelMessage, SettingsPanelComponent, SettingsPanelMessage,
        },
        rss_feed::RssFeedComponentMessage::UpdateRssFeed,
        style::{
            button::{ButtonState, ButtonStyle, DownloadButtonStyle},
            container::ContainerStyle,
        },
        subscriptions,
        views::Action,
        widget::*,
    },
    launcher_update::LauncherUpdate,
    profiles::Profile,
};

use iced::{
    Alignment, Command, Length,
    alignment::Horizontal,
    widget::{button, column, container, image::Handle, row, text},
};
use std::time::Duration;

/// How long each background image stays up before rotating to the next one.
const BACKGROUND_ROTATION_INTERVAL: Duration = Duration::from_secs(12);

/// How often to silently re-check for a new game version while the app is open and
/// idle at the main menu. This is a subscription tied to the running app, not a
/// background OS process - it stops firing the moment the app is closed.
const UPDATE_CHECK_INTERVAL: Duration = Duration::from_secs(15 * 60);

/// State of the mandatory "the launcher itself needs updating" prompt. Unlike the
/// game's own update prompt (see `GamePanelComponent`), there's no "not now" - the
/// user can't reach the game panel at all until this resolves, since an out-of-date
/// launcher may not even be able to fetch a working game manifest.
#[derive(Debug, Clone)]
enum LauncherUpdateState {
    /// Waiting for the user to press "Update now".
    Prompt(LauncherUpdate),
    /// Downloading/verifying/applying - no interaction possible.
    Applying(LauncherUpdate),
    /// Applying failed (e.g. offline mid-download); the user can retry.
    Failed(LauncherUpdate, String),
}

#[derive(Default, Debug, Clone)]
pub struct DefaultView {
    changelog_panel_component: ChangelogPanelComponent,
    announcement_panel_component: AnnouncementPanelComponent,
    logo_panel_component: LogoPanelComponent,
    game_panel_component: GamePanelComponent,
    news_panel_component: NewsPanelComponent,
    settings_panel_component: SettingsPanelComponent,
    server_browser_panel_component: ServerBrowserPanelComponent,
    show_settings: bool,
    show_server_browser: bool,
    background_index: usize,
    launcher_update: Option<LauncherUpdateState>,
    /// A brief "updated successfully" notice - not blocking (unlike the modals
    /// above), just an extra row in the left sidebar that clears itself after a few
    /// seconds. Shown for both a game auto-update (see `GamePanelComponent`) and a
    /// launcher auto-update (detected via `Profile::pending_launcher_update_notice`
    /// on the launch right after it happened).
    toast: Option<String>,
}

/// How long the "updated successfully" toast stays up before clearing itself.
const TOAST_DURATION: Duration = Duration::from_secs(5);

#[derive(Clone, Debug)]
pub enum DefaultViewMessage {
    // Messages
    Action(Action),
    Query,

    LauncherUpdateFound(Option<LauncherUpdate>),
    LauncherUpdateConfirm,
    LauncherUpdateApplied(LauncherUpdate, Result<(), String>),

    ShowToast(String),
    DismissToast,

    // User Interactions
    Interaction(Interaction),

    BackgroundTick,

    // Panel-specific messages
    GamePanel(GamePanelMessage),
    ChangelogPanel(ChangelogPanelMessage),
    AnnouncementPanel(AnnouncementPanelMessage),
    NewsPanel(NewsPanelMessage),
    SettingsPanel(SettingsPanelMessage),
    ServerBrowserPanel(ServerBrowserPanelMessage),
}

#[derive(Debug, Clone)]
pub enum Interaction {
    SettingsPressed,
    ToggleServerBrowser,
    OpenURL(String),
}

impl DefaultView {
    pub fn subscription(&self) -> iced::Subscription<DefaultViewMessage> {
        iced::Subscription::batch(
            IntoIterator::into_iter([
                Some(
                    self.game_panel_component
                        .subscription()
                        .map(DefaultViewMessage::GamePanel),
                ),
                self.show_server_browser.then_some(
                    subscriptions::repeat_message::stream(
                        SERVER_BROWSER_PING_REFRESH,
                        DefaultViewMessage::ServerBrowserPanel(
                            ServerBrowserPanelMessage::RefreshPing,
                        ),
                    ),
                ),
                Some(
                    iced::time::every(BACKGROUND_ROTATION_INTERVAL)
                        .map(|_| DefaultViewMessage::BackgroundTick),
                ),
                Some(iced::time::every(UPDATE_CHECK_INTERVAL).map(|_| {
                    DefaultViewMessage::GamePanel(GamePanelMessage::PeriodicUpdateCheck)
                })),
            ])
            .flatten(),
        )
    }

    pub fn view<'a>(
        &'a self,
        active_profile: &'a Profile,
    ) -> Element<'a, DefaultViewMessage> {
        let Self {
            changelog_panel_component,
            announcement_panel_component,
            news_panel_component,
            logo_panel_component,
            game_panel_component,
            settings_panel_component,
            server_browser_panel_component,
            ..
        } = self;

        let mut left_column =
            column![].push(container(logo_panel_component.view()).height(Length::Fill));
        if self.show_settings {
            left_column = left_column.push(
                container(settings_panel_component.view(active_profile))
                    .height(Length::Shrink),
            );
        }
        left_column = left_column.push(
            container(game_panel_component.view(active_profile)).height(Length::Shrink),
        );
        if let Some(message) = &self.toast {
            left_column = left_column.push(container(toast_banner(message)).padding(10));
        }

        let left = container(left_column)
            .height(Length::Fill)
            .width(Length::Fixed(360.0));

        let mut main_row = row![].push(left);

        if !self.show_server_browser {
            let middle = container(
                column![]
                    .push(
                        container(announcement_panel_component.view())
                            .height(Length::Shrink),
                    )
                    .push(
                        container(changelog_panel_component.view()).height(Length::Fill),
                    ),
            )
            .height(Length::Fill)
            .width(Length::Fill);
            let right = container(news_panel_component.view())
                .height(Length::Fill)
                .width(Length::Fixed(248.0));

            main_row = main_row.push(middle).push(right);
        } else {
            let server_browser = container(server_browser_panel_component.view())
                .height(Length::Fill)
                .width(Length::Fill);
            main_row = main_row.push(server_browser);
        }

        // One background image spans the whole window - the left/right sidebars are
        // transparent windows onto it, and the opaque middle (changelog) panel just
        // covers its own slice of it, same as any other content sitting on top.
        let background_bytes =
            BACKGROUND_IMAGES[self.background_index % BACKGROUND_IMAGES.len()];

        let content = layered(
            centered_cover(Handle::from_memory(background_bytes)),
            container(main_row).width(Length::Fill).height(Length::Fill),
        );

        // The "new game version available" prompt is a third layer on top of
        // everything else - it blocks interaction with the rest of the window while
        // it's up, same trick used to keep the background photo from stealing clicks.
        let content = match game_panel_component.update_prompt() {
            Some(prompt) => layered(content, prompt.map(DefaultViewMessage::GamePanel)),
            None => content,
        };

        // The mandatory launcher-update dialog sits above even the game's own
        // prompt - an out-of-date launcher may not be able to fetch a working game
        // manifest at all, so it takes priority.
        match &self.launcher_update {
            Some(state) => layered(content, launcher_update_dialog(state)),
            None => content,
        }
    }

    pub fn update(
        &mut self,
        msg: DefaultViewMessage,
        active_profile: &Profile,
    ) -> Command<DefaultViewMessage> {
        match msg {
            // Messages
            // Will be handled by main view
            DefaultViewMessage::Action(_) => {},
            DefaultViewMessage::BackgroundTick => {
                self.background_index =
                    (self.background_index + 1) % BACKGROUND_IMAGES.len();
            },
            DefaultViewMessage::Query => {
                let api_version_url = active_profile.api_version_url();
                let announcement_url = active_profile.announcement_url();

                // A launcher auto-update applied right before this process started -
                // let the user know it went through, then clear the marker so it
                // doesn't show again next launch.
                let mut commands = vec![];
                if let Some(old_version) =
                    active_profile.pending_launcher_update_notice.clone()
                {
                    let mut cleared_profile = active_profile.clone();
                    cleared_profile.pending_launcher_update_notice = None;
                    commands.push(Command::perform(
                        async { Action::UpdateProfile(cleared_profile) },
                        DefaultViewMessage::Action,
                    ));
                    commands.push(Command::perform(async {}, move |_| {
                        DefaultViewMessage::ShowToast(format!(
                            "Launcher updated from v{old_version} to v{}",
                            env!("CARGO_PKG_VERSION")
                        ))
                    }));
                }

                commands.extend([
                    Command::perform(NewsPanelComponent::load_news(), |update| {
                        DefaultViewMessage::NewsPanel(NewsPanelMessage::RssUpdate(
                            UpdateRssFeed(update),
                        ))
                    }),
                    Command::perform(
                        ChangelogPanelComponent::load_changelog(),
                        |update| {
                            DefaultViewMessage::ChangelogPanel(
                                ChangelogPanelMessage::LoadChangelog(update),
                            )
                        },
                    ),
                    Command::perform(ServerBrowserPanelComponent::fetch(), |update| {
                        DefaultViewMessage::ServerBrowserPanel(
                            ServerBrowserPanelMessage::UpdateServerList(update),
                        )
                    }),
                    Command::perform(
                        AnnouncementPanelComponent::fetch(
                            api_version_url,
                            announcement_url,
                        ),
                        |update| {
                            DefaultViewMessage::AnnouncementPanel(
                                AnnouncementPanelMessage::FetchAnnouncement(update),
                            )
                        },
                    ),
                    Command::perform(
                        Channels::fetch(active_profile.channel_url()),
                        |channels| {
                            DefaultViewMessage::SettingsPanel(
                                SettingsPanelMessage::ChannelsLoaded(channels),
                            )
                        },
                    ),
                    Command::perform(
                        crate::launcher_update::check_for_update(),
                        DefaultViewMessage::LauncherUpdateFound,
                    ),
                    Command::perform(async {}, |_| {
                        DefaultViewMessage::GamePanel(GamePanelMessage::StartUpdate)
                    }),
                ]);
                return Command::batch(commands);
            },

            DefaultViewMessage::GamePanel(msg) => {
                if let Some(command) =
                    self.game_panel_component.update(msg, active_profile)
                {
                    return command;
                }
            },
            DefaultViewMessage::ChangelogPanel(msg) => {
                if let Some(command) = self.changelog_panel_component.update(msg) {
                    return command;
                }
            },
            DefaultViewMessage::AnnouncementPanel(msg) => {
                if let Some(command) = self.announcement_panel_component.update(msg) {
                    return command;
                }
            },
            DefaultViewMessage::NewsPanel(msg) => {
                if let Some(command) = self.news_panel_component.update(msg) {
                    return command;
                }
            },

            DefaultViewMessage::SettingsPanel(msg) => {
                if let Some(command) =
                    self.settings_panel_component.update(msg, active_profile)
                {
                    return command;
                }
            },
            DefaultViewMessage::ServerBrowserPanel(msg) => {
                if let Some(command) = self.server_browser_panel_component.update(msg) {
                    return command;
                }
            },

            DefaultViewMessage::LauncherUpdateFound(Some(update)) => {
                if active_profile.auto_update {
                    return self.apply_launcher_update(update, active_profile);
                }
                self.launcher_update = Some(LauncherUpdateState::Prompt(update));
            },
            DefaultViewMessage::LauncherUpdateFound(None) => {},
            DefaultViewMessage::LauncherUpdateConfirm => {
                if let Some(
                    LauncherUpdateState::Prompt(update)
                    | LauncherUpdateState::Failed(update, _),
                ) = self.launcher_update.clone()
                {
                    return self.apply_launcher_update(update, active_profile);
                }
            },
            DefaultViewMessage::LauncherUpdateApplied(update, result) => {
                // Success doesn't actually reach here in practice - applying an
                // update replaces/relaunches the process instead of returning. This
                // only fires on failure (e.g. offline mid-download), so the user can
                // retry.
                self.launcher_update = match result {
                    Ok(()) => None,
                    Err(reason) => Some(LauncherUpdateState::Failed(update, reason)),
                };
            },

            DefaultViewMessage::ShowToast(message) => {
                self.toast = Some(message);
                return Command::perform(tokio::time::sleep(TOAST_DURATION), |_| {
                    DefaultViewMessage::DismissToast
                });
            },
            DefaultViewMessage::DismissToast => {
                self.toast = None;
            },

            // User Interaction
            DefaultViewMessage::Interaction(interaction) => match interaction {
                Interaction::SettingsPressed => {
                    self.show_settings = !self.show_settings;
                },
                Interaction::ToggleServerBrowser => {
                    self.show_server_browser = !self.show_server_browser;

                    // If toggling the server browser panel resulted in it being hidden,
                    // deselect the selected server to switch the
                    // Launch button back to saying "Launch" instead of "Connect to
                    // selected server"
                    if !self.show_server_browser {
                        return Command::perform(async {}, |_| {
                            DefaultViewMessage::ServerBrowserPanel(
                                ServerBrowserPanelMessage::SelectServerEntry(None),
                            )
                        });
                    }
                },
                Interaction::OpenURL(url) => {
                    if let Err(e) = opener::open(url) {
                        tracing::error!(
                            "Failed to open gitlab changelog website: {:?}",
                            e
                        );
                    }
                },
            },
        }

        Command::none()
    }

    fn apply_launcher_update(
        &mut self,
        update: LauncherUpdate,
        active_profile: &Profile,
    ) -> Command<DefaultViewMessage> {
        self.launcher_update = Some(LauncherUpdateState::Applying(update.clone()));
        let update_for_result = update.clone();
        let profile = active_profile.clone();
        Command::perform(
            crate::launcher_update::apply(update, profile),
            move |result| {
                DefaultViewMessage::LauncherUpdateApplied(
                    update_for_result.clone(),
                    result.map_err(|e| e.to_string()),
                )
            },
        )
    }
}

/// The mandatory "the launcher needs updating" dialog. There's no dismiss button -
/// `default.rs`'s `view()` layers this on top of the whole window, same as the game's
/// own update prompt, so it blocks all interaction until resolved.
fn launcher_update_dialog(
    state: &LauncherUpdateState,
) -> Element<'static, DefaultViewMessage> {
    let (heading, body, action): (
        &str,
        String,
        Option<Element<'static, DefaultViewMessage>>,
    ) = match state {
        LauncherUpdateState::Prompt(update) => (
            "Launcher update required",
            format!(
                "A new version of the launcher ({}) is available. You need to update \
                 before you can play or download the game.",
                update.version
            ),
            Some(
                button(
                    text("Update now")
                        .font(crate::assets::POPPINS_BOLD_FONT)
                        .size(14),
                )
                .style(ButtonStyle::Download(DownloadButtonStyle::Update(
                    ButtonState::Enabled,
                )))
                .padding([10, 24])
                .on_press(DefaultViewMessage::LauncherUpdateConfirm)
                .into(),
            ),
        ),
        LauncherUpdateState::Applying(update) => (
            "Updating the launcher...",
            format!("Downloading and installing version {}.", update.version),
            None,
        ),
        LauncherUpdateState::Failed(update, reason) => (
            "Launcher update failed",
            format!(
                "Couldn't update to version {}: {reason}. Check your connection and try \
                 again.",
                update.version
            ),
            Some(
                button(
                    text("Retry")
                        .font(crate::assets::POPPINS_BOLD_FONT)
                        .size(14),
                )
                .style(ButtonStyle::Download(DownloadButtonStyle::Update(
                    ButtonState::Enabled,
                )))
                .padding([10, 24])
                .on_press(DefaultViewMessage::LauncherUpdateConfirm)
                .into(),
            ),
        ),
    };

    let mut card = column![]
        .align_items(Alignment::Center)
        .spacing(16)
        .padding(24)
        .push(
            text(heading)
                .font(crate::assets::POPPINS_BOLD_FONT)
                .size(20),
        )
        .push(
            text(body)
                .size(14)
                .horizontal_alignment(Horizontal::Center)
                .width(Length::Fixed(340.0)),
        );
    if let Some(action) = action {
        card = card.push(action);
    }

    container(
        container(card)
            .style(ContainerStyle::ModalDialog)
            .width(Length::Fixed(380.0)),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .align_x(Horizontal::Center)
    .align_y(iced::alignment::Vertical::Center)
    .style(ContainerStyle::ModalBackdrop)
    .into()
}

/// A brief, non-blocking "updated successfully" notice - unlike the modals above,
/// this is just an extra row in the normal layout, not a full-window overlay.
fn toast_banner(message: &str) -> Element<'_, DefaultViewMessage> {
    container(
        text(message)
            .size(12)
            .horizontal_alignment(Horizontal::Center)
            .width(Length::Fill),
    )
    .padding(8)
    .width(Length::Fill)
    .style(ContainerStyle::Toast)
    .into()
}
