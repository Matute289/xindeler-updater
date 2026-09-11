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
        subscriptions,
        views::Action,
        widget::*,
    },
    profiles::Profile,
};

use iced::{
    Command, Length,
    widget::{column, container, image::Handle, row},
};
use std::time::Duration;

/// How long each background image stays up before rotating to the next one.
const BACKGROUND_ROTATION_INTERVAL: Duration = Duration::from_secs(12);

/// How often to silently re-check for a new game version while the app is open and
/// idle at the main menu. This is a subscription tied to the running app, not a
/// background OS process - it stops firing the moment the app is closed.
const UPDATE_CHECK_INTERVAL: Duration = Duration::from_secs(15 * 60);

#[cfg(windows)]
use crate::gui::Result;

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
}

#[cfg(debug_assertions)]
impl DefaultView {
    /// Builds a `DefaultView` with the game panel pre-set to a specific mock
    /// state, for `--mock-state` manual visual testing.
    pub fn with_mock_game_panel_state(
        mock_state: crate::cli::MockGameState,
        active_profile: &Profile,
    ) -> Self {
        Self {
            game_panel_component: GamePanelComponent::mock(mock_state, active_profile),
            ..Default::default()
        }
    }
}

#[derive(Clone, Debug)]
pub enum DefaultViewMessage {
    // Messages
    Action(Action),
    Query,

    #[cfg(windows)]
    LauncherUpdate(Result<Option<self_update::update::Release>>),

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

        // The "new version available" prompt is a third layer on top of everything
        // else - it blocks interaction with the rest of the window while it's up,
        // same trick used to keep the background photo from stealing clicks.
        match game_panel_component.update_prompt() {
            Some(prompt) => layered(content, prompt.map(DefaultViewMessage::GamePanel)),
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
                return Command::batch(vec![
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
                    #[cfg(windows)]
                    Command::perform(
                        async { tokio::task::block_in_place(crate::windows::query) },
                        DefaultViewMessage::LauncherUpdate,
                    ),
                    Command::perform(async {}, |_| {
                        DefaultViewMessage::GamePanel(GamePanelMessage::StartUpdate)
                    }),
                ]);
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

            #[cfg(windows)]
            DefaultViewMessage::LauncherUpdate(update) => {
                if let Ok(Some(release)) = update {
                    return Command::perform(
                        async { Action::LauncherUpdate(release) },
                        DefaultViewMessage::Action,
                    );
                }
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
}
