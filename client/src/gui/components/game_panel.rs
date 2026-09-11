use crate::{
    assets::{POPPINS_BOLD_FONT, POPPINS_MEDIUM_FONT, SETTINGS_ICON},
    gui::{
        custom_widgets::{heading_with_rule, modal_shell},
        style::{GOLD_500, button::ButtonStyle, text::TextStyle},
        subscriptions,
        views::{
            Action,
            default::{
                DefaultViewMessage,
                Interaction::{self, SettingsPressed},
            },
        },
        widget::*,
    },
    io::ProcessUpdate,
    logger::{pretty_bytes, redirect_voxygen_log},
    profiles::Profile,
    update::{Progress, State},
};
use iced::{
    Alignment, Command, Length,
    alignment::{Horizontal, Vertical},
    widget::{
        button, column, container, image, image::Handle, progress_bar, row, text,
        text::LineHeight, tooltip, tooltip::Position,
    },
};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;

use crate::gui::style::container::ContainerStyle;
use tracing::debug;

#[derive(Debug, Clone)]
pub enum GamePanelMessage {
    ProcessUpdate(ProcessUpdate),
    DownloadProgress(Box<Option<Progress>>),
    PlayPressed,
    CancelDownload,
    ServerBrowserServerChanged(Option<String>),
    StartUpdate,
    /// Fired on a timer while the app is open and idle - a no-op unless the game
    /// is currently installed and playable, in which case it silently re-checks
    /// for a new version the same way the startup check does.
    PeriodicUpdateCheck,
    /// "Actualizar" was pressed, either in the new-version prompt or next to the
    /// Play button after the prompt was dismissed.
    ConfirmUpdate,
    /// "Ahora no" was pressed in the new-version prompt.
    DismissUpdatePrompt,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DownloadButtonState {
    Checking,
    WaitForConfirm,
    InProgress,
}

#[derive(Clone)]
pub enum GamePanelState {
    Updating {
        astate: Arc<Mutex<Option<State>>>,
        btnstate: DownloadButtonState,
    },
    /// A new version was found for a profile that's already installed and playable.
    /// The "update now?" prompt is shown on top of everything else; the game
    /// underneath stays on the old, already-installed version until the user
    /// answers.
    UpdatePrompt {
        astate: Arc<Mutex<Option<State>>>,
        version: String,
    },
    /// The user dismissed the prompt above. The old version is still playable, but
    /// an "Update" button sits next to Play so they can start the update whenever
    /// they want.
    UpdateAvailable {
        astate: Arc<Mutex<Option<State>>>,
        version: String,
    },
    ReadyToPlay,
    Playing(Box<Profile>),
    Offline(bool),
    Retry,
}

#[derive(Debug, Clone)]
pub struct GamePanelComponent {
    state: GamePanelState,
    download_progress: Option<Progress>,
    selected_server_browser_address: Option<String>,
    /// The version the update check last found on the server, regardless of whether
    /// it's been downloaded yet. `active_profile.version` only reflects the last
    /// *successfully installed* version, so before the user confirms a download
    /// there's otherwise no way for the UI to say what's actually about to be
    /// installed - see the "why does it say v0.26.0 when v0.26.1 is out" confusion
    /// this fixes.
    available_version: Option<String>,
    /// Set when the current download was auto-started (`Profile::auto_update_game`, no
    /// prompt shown) - consulted once at `Progress::Successful` to decide whether to
    /// show a "game updated successfully" toast, then cleared.
    auto_triggered_download: bool,
}

impl std::fmt::Debug for GamePanelState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GamePanelState::Updating { .. } => write!(f, "GamePanelState::Updating"),
            GamePanelState::UpdatePrompt { .. } => {
                write!(f, "GamePanelState::UpdatePrompt")
            },
            GamePanelState::UpdateAvailable { .. } => {
                write!(f, "GamePanelState::UpdateAvailable")
            },
            GamePanelState::ReadyToPlay => write!(f, "GamePanelState::ReadyToPlay"),
            GamePanelState::Playing(_) => write!(f, "GamePanelState::Playing"),
            GamePanelState::Offline(_) => write!(f, "GamePanelState::Offline"),
            GamePanelState::Retry => write!(f, "GamePanelState::Retry"),
        }
    }
}

impl Default for GamePanelComponent {
    fn default() -> Self {
        Self {
            state: GamePanelState::ReadyToPlay,
            download_progress: None,
            selected_server_browser_address: None,
            available_version: None,
            auto_triggered_download: false,
        }
    }
}

#[cfg(debug_assertions)]
impl GamePanelComponent {
    /// Builds a component already in the given mock state, for `--mock-state`
    /// manual visual testing without a real install/server.
    ///
    /// `astate` is filled with a real `State::ToBeEvaluated` wrapping the current
    /// profile rather than left empty, so clicking Update/Download in a mocked
    /// state does a real re-check instead of panicking on the empty-Mutex
    /// `.expect(...)` the real confirm flow relies on.
    pub fn mock(mock_state: crate::cli::MockGameState, active_profile: &Profile) -> Self {
        use crate::cli::MockGameState;

        const MOCK_VERSION: &str = "0.99.0-mock";
        let fresh_astate = || {
            Arc::new(Mutex::new(Some(State::ToBeEvaluated(
                active_profile.clone(),
            ))))
        };

        let state = match mock_state {
            MockGameState::Ready => GamePanelState::ReadyToPlay,
            MockGameState::Checking => GamePanelState::Updating {
                astate: fresh_astate(),
                btnstate: DownloadButtonState::Checking,
            },
            MockGameState::WaitForConfirm => GamePanelState::Updating {
                astate: fresh_astate(),
                btnstate: DownloadButtonState::WaitForConfirm,
            },
            MockGameState::UpdatePrompt => GamePanelState::UpdatePrompt {
                astate: fresh_astate(),
                version: MOCK_VERSION.to_owned(),
            },
            MockGameState::UpdateAvailable => GamePanelState::UpdateAvailable {
                astate: fresh_astate(),
                version: MOCK_VERSION.to_owned(),
            },
            MockGameState::OfflinePlayable => GamePanelState::Offline(true),
            MockGameState::OfflineUnplayable => GamePanelState::Offline(false),
            MockGameState::Retry => GamePanelState::Retry,
        };

        let available_version = matches!(
            mock_state,
            MockGameState::WaitForConfirm
                | MockGameState::UpdatePrompt
                | MockGameState::UpdateAvailable
        )
        .then(|| MOCK_VERSION.to_owned());

        Self {
            state,
            available_version,
            ..Default::default()
        }
    }
}

impl GamePanelComponent {
    pub fn subscription(&self) -> iced::Subscription<GamePanelMessage> {
        match &self.state {
            GamePanelState::Playing(profile) => subscriptions::process::stream(
                profile.as_ref().clone(),
                self.selected_server_browser_address.clone(),
            )
            .map(GamePanelMessage::ProcessUpdate),
            _ => iced::Subscription::none(),
        }
    }

    fn trigger_next_state(
        state: State,
        empty_arc_state: Arc<Mutex<Option<State>>>,
        dstate: DownloadButtonState,
    ) -> (Option<GamePanelState>, Option<Command<DefaultViewMessage>>) {
        (
            Some(GamePanelState::Updating {
                astate: empty_arc_state.clone(),
                btnstate: dstate.clone(),
            }),
            Some(Command::perform(
                async move {
                    let start_time = Instant::now();
                    let mut last_progress = None;
                    let mut lstate = state;
                    // ICED is really slow, so we have to do multiple steps
                    while start_time.elapsed() < Duration::from_millis(30) {
                        match lstate.progress().await {
                            Some((progress, state)) => {
                                lstate = state;
                                last_progress = Some(progress);
                                if matches!(
                                    last_progress,
                                    Some(Progress::ReadyToSync { .. })
                                ) {
                                    // wait for user input!
                                    break;
                                }
                            },
                            None => {
                                return last_progress;
                            },
                        }
                    }
                    *empty_arc_state.lock().await = Some(lstate);
                    last_progress
                },
                |progress| {
                    DefaultViewMessage::GamePanel(GamePanelMessage::DownloadProgress(
                        Box::new(progress),
                    ))
                },
            )),
        )
    }

    pub fn update(
        &mut self,
        msg: GamePanelMessage,
        active_profile: &Profile,
    ) -> Option<Command<DefaultViewMessage>> {
        let (next_state, command) = match msg {
            GamePanelMessage::PlayPressed => match &self.state {
                GamePanelState::ReadyToPlay => (
                    Some(GamePanelState::Playing(Box::new(active_profile.clone()))),
                    None,
                ),
                GamePanelState::Retry => (
                    None,
                    Some(Command::perform(async {}, |_| {
                        DefaultViewMessage::GamePanel(GamePanelMessage::StartUpdate)
                    })),
                ),
                GamePanelState::Offline(available) => {
                    match available {
                        // Play offline
                        true => (
                            Some(GamePanelState::Playing(Box::new(
                                active_profile.clone(),
                            ))),
                            None,
                        ),
                        // Retry
                        false => {
                            // The game has never been downloaded so the only option is to
                            // retry the download
                            (
                                None,
                                Some(Command::perform(async {}, |_| {
                                    DefaultViewMessage::GamePanel(
                                        GamePanelMessage::StartUpdate,
                                    )
                                })),
                            )
                        },
                    }
                },
                GamePanelState::Updating { btnstate, astate }
                    if *btnstate == DownloadButtonState::WaitForConfirm =>
                {
                    let state = {
                        let mut l = astate.blocking_lock();
                        l.take().expect("impossible, should always be filled")
                    };
                    Self::trigger_next_state(
                        state,
                        astate.clone(),
                        DownloadButtonState::InProgress,
                    )
                },
                GamePanelState::UpdateAvailable { .. } => (
                    Some(GamePanelState::Playing(Box::new(active_profile.clone()))),
                    None,
                ),
                GamePanelState::Updating { .. }
                | GamePanelState::UpdatePrompt { .. }
                | GamePanelState::Playing(..) => (None, None),
            },
            GamePanelMessage::ConfirmUpdate => match &self.state {
                GamePanelState::UpdatePrompt { astate, .. }
                | GamePanelState::UpdateAvailable { astate, .. } => {
                    let state = {
                        let mut l = astate.blocking_lock();
                        l.take().expect("impossible, should always be filled")
                    };
                    Self::trigger_next_state(
                        state,
                        astate.clone(),
                        DownloadButtonState::InProgress,
                    )
                },
                _ => (None, None),
            },
            GamePanelMessage::DismissUpdatePrompt => match &self.state {
                GamePanelState::UpdatePrompt { astate, version } => (
                    Some(GamePanelState::UpdateAvailable {
                        astate: astate.clone(),
                        version: version.clone(),
                    }),
                    None,
                ),
                _ => (None, None),
            },
            GamePanelMessage::CancelDownload => match &self.state {
                GamePanelState::Updating { .. } => {
                    tracing::info!("Download cancelled by user");
                    (Some(GamePanelState::Retry), None)
                },
                _ => (None, None),
            },
            GamePanelMessage::StartUpdate => {
                let state = State::ToBeEvaluated(active_profile.clone());

                let astate = Arc::new(Mutex::new(None));
                Self::trigger_next_state(state, astate, DownloadButtonState::Checking)
            },
            GamePanelMessage::PeriodicUpdateCheck => {
                if matches!(self.state, GamePanelState::ReadyToPlay) {
                    let state = State::ToBeEvaluated(active_profile.clone());
                    let astate = Arc::new(Mutex::new(None));
                    Self::trigger_next_state(state, astate, DownloadButtonState::Checking)
                } else {
                    // Don't interrupt an active download, an ongoing play session, or
                    // a prompt/offline/retry state the user hasn't resolved yet.
                    (None, None)
                }
            },
            GamePanelMessage::DownloadProgress(progress) => {
                let next = match &progress.as_ref() {
                    Some(Progress::Errored(e)) => {
                        tracing::error!("Download failed with: {e}");
                        (Some(GamePanelState::Retry), None)
                    },
                    Some(Progress::Successful(profile)) => {
                        let profile = profile.clone();
                        let version = profile.version.clone().unwrap_or_default();
                        let mut commands = vec![Command::perform(
                            async move { Action::UpdateProfile(profile) },
                            DefaultViewMessage::Action,
                        )];
                        if self.auto_triggered_download {
                            self.auto_triggered_download = false;
                            commands.push(Command::perform(async {}, move |_| {
                                // `version` already comes with a leading "v".
                                DefaultViewMessage::ShowToast(format!(
                                    "Game updated to {version}"
                                ))
                            }));
                        }
                        (
                            Some(GamePanelState::ReadyToPlay),
                            Some(Command::batch(commands)),
                        )
                    },
                    Some(Progress::Offline) => (
                        Some(GamePanelState::Offline(active_profile.installed())),
                        None,
                    ),
                    Some(Progress::Incomplete { .. }) => {
                        if let GamePanelState::Updating { astate, btnstate } = &self.state
                        {
                            let state = {
                                let mut l = astate.blocking_lock();
                                l.take()
                            };
                            match state {
                                Some(state) => Self::trigger_next_state(
                                    state,
                                    astate.clone(),
                                    btnstate.clone(),
                                ),
                                None => {
                                    tracing::warn!("Wrong State"); // might happen if there is a click right between this and the resulting command
                                    (None, None)
                                },
                            }
                        } else {
                            tracing::warn!("Wrong State");
                            (None, None)
                        }
                    },
                    Some(Progress::ReadyToSync { version }) => {
                        tracing::debug!(?version, "Need to confirm the update");
                        self.available_version = Some(version.clone());
                        match &self.state {
                            GamePanelState::Updating { astate, .. }
                                if active_profile.auto_update_game =>
                            {
                                // Skip the prompt entirely and start downloading -
                                // Profile::auto_update_game opted into this.
                                self.auto_triggered_download = true;
                                let state = {
                                    let mut l = astate.blocking_lock();
                                    l.take().expect("impossible, should always be filled")
                                };
                                Self::trigger_next_state(
                                    state,
                                    astate.clone(),
                                    DownloadButtonState::InProgress,
                                )
                            },
                            GamePanelState::Updating { astate, .. }
                                if active_profile.installed() =>
                            {
                                // Already playable on the old version - ask before
                                // disrupting anything, rather than just swapping the
                                // Launch button for a Download one.
                                (
                                    Some(GamePanelState::UpdatePrompt {
                                        astate: astate.clone(),
                                        version: version.clone(),
                                    }),
                                    None,
                                )
                            },
                            GamePanelState::Updating { astate, .. } => {
                                // Nothing installed yet, there's no "play the old
                                // version" option to offer - just ask to download.
                                (
                                    Some(GamePanelState::Updating {
                                        astate: astate.clone(),
                                        btnstate: DownloadButtonState::WaitForConfirm,
                                    }),
                                    None,
                                )
                            },
                            _ => (None, None),
                        }
                    },
                    None => (None, None),
                };
                self.download_progress = progress.as_ref().clone();
                next
            },
            // TODO: Move this out of GamePanelComponent? This code handles redirecting
            // voxygen output to XindelerUpdater's log output
            GamePanelMessage::ProcessUpdate(update) => match update {
                ProcessUpdate::Line(msg) => {
                    redirect_voxygen_log(&msg);
                    (None, None)
                },
                ProcessUpdate::Exit(code) => {
                    debug!("Xindeler exited with {}", code);
                    (
                        Some(GamePanelState::Retry),
                        Some(Command::perform(async {}, |_| {
                            DefaultViewMessage::GamePanel(GamePanelMessage::StartUpdate)
                        })),
                    )
                },
                ProcessUpdate::Error(err) => {
                    tracing::error!(
                        "Failed to receive an update from Xindeler process! {}",
                        err
                    );
                    (Some(GamePanelState::Retry), None)
                },
            },
            GamePanelMessage::ServerBrowserServerChanged(server_address) => {
                self.selected_server_browser_address = server_address;
                (None, None)
            },
        };

        if let Some(state) = next_state {
            self.set_state(state);
        }

        command
    }

    pub fn view(&self, active_profile: &Profile) -> Element<'_, DefaultViewMessage> {
        let mut version_string = "Pre-Alpha".to_owned();
        if let Some(version) = &active_profile.version {
            // `version` already comes formatted with a leading "v" (e.g. "v0.26.1"),
            // so no extra "v" goes here - see the "vv0.26.1" bug this fixes.
            version_string.push_str(&format!(" · {version}"));
        }
        let available_notice = self
            .available_version
            .as_ref()
            .filter(|available| active_profile.version.as_ref() != Some(*available));

        let mut version_row = row![]
            .spacing(6)
            .align_items(Alignment::Center)
            .push(text(version_string).size(11).style(TextStyle::Muted));
        if let Some(available) = available_notice {
            version_row = version_row.push(
                text(format!("{available} available"))
                    .size(11)
                    .style(TextStyle::Accent),
            );
        }

        column![]
            .push(heading_with_rule::<DefaultViewMessage>("Game Version"))
            .push(
                container(
                    row![]
                        .height(Length::Fixed(30.0))
                        .push(
                            container(version_row)
                                .align_y(Vertical::Bottom)
                                .width(Length::Fill)
                                .height(Length::Fill),
                        )
                        .push(
                            tooltip(
                                container(
                                    button(image(Handle::from_memory(
                                        SETTINGS_ICON.to_vec(),
                                    )))
                                    .style(ButtonStyle::Icon)
                                    .on_press(
                                        DefaultViewMessage::Interaction(SettingsPressed),
                                    ),
                                )
                                .center_y(),
                                text("Settings").size(14),
                                Position::Left,
                            )
                            .style(ContainerStyle::Tooltip)
                            .gap(5),
                        ),
                )
                .padding([0, 20]),
            )
            .push(
                container(self.download_area())
                    .width(Length::Fill)
                    .padding([10, 20, 20, 20]),
            )
            .into()
    }

    /// The "a new version is available, update now?" dialog, when there is one to
    /// show. `default.rs` layers this on top of the whole window so it blocks
    /// interaction with everything else until answered.
    pub fn update_prompt(&self) -> Option<Element<'static, GamePanelMessage>> {
        match &self.state {
            GamePanelState::UpdatePrompt { version, .. } => {
                Some(update_prompt_dialog(version))
            },
            _ => None,
        }
    }
}

fn update_prompt_dialog(version: &str) -> Element<'static, GamePanelMessage> {
    let title = format!("Version {version} is available");
    let body = text(
        "You can install it now, or keep playing on your current version and update \
         later.",
    )
    .size(14)
    .style(TextStyle::Secondary)
    .into();

    let actions = row![]
        .spacing(10)
        .push(
            button(text("Not now").font(POPPINS_MEDIUM_FONT).size(14))
                .style(ButtonStyle::Ghost)
                .padding([10, 18])
                .on_press(GamePanelMessage::DismissUpdatePrompt),
        )
        .push(
            button(text("Update now").font(POPPINS_BOLD_FONT).size(14))
                .style(ButtonStyle::Primary)
                .padding([10, 22])
                .on_press(GamePanelMessage::ConfirmUpdate),
        )
        .into();

    modal_shell(GOLD_500, "GAME UPDATE", title, body, Some(actions))
}

impl GamePanelComponent {
    fn set_state(&mut self, state: GamePanelState) {
        use GamePanelState::*;
        let same = match &self.state {
            Updating { .. } => matches!(state, Updating { .. }),
            UpdatePrompt { .. } => matches!(state, UpdatePrompt { .. }),
            UpdateAvailable { .. } => matches!(state, UpdateAvailable { .. }),
            ReadyToPlay => matches!(state, ReadyToPlay),
            Playing(_) => matches!(state, Playing(_)),
            Offline(_) => matches!(state, Offline(_)),
            Retry => matches!(state, Retry),
        };
        if !same {
            debug!("GamePanel state: {:?} -> {:?}", self.state, state);
        }
        self.state = state;
    }

    fn download_area(&self) -> Element<'_, DefaultViewMessage> {
        match &self.state {
            GamePanelState::UpdateAvailable { version, .. } => {
                let play_button = button(
                    text("PLAY")
                        .font(POPPINS_BOLD_FONT)
                        .size(24)
                        .horizontal_alignment(Horizontal::Center)
                        .vertical_alignment(Vertical::Center)
                        .width(Length::Fill),
                )
                .style(ButtonStyle::Primary)
                .width(Length::FillPortion(2))
                .height(Length::Fixed(72.0))
                .on_press(DefaultViewMessage::GamePanel(GamePanelMessage::PlayPressed));

                let update_button = button(
                    column![]
                        .align_items(Alignment::Center)
                        .width(Length::Fill)
                        .spacing(2)
                        .push(text("Update").font(POPPINS_MEDIUM_FONT).size(15))
                        .push(text(version.clone()).size(11).style(TextStyle::Secondary)),
                )
                .style(ButtonStyle::Accent)
                .width(Length::FillPortion(1))
                .height(Length::Fixed(72.0))
                .on_press(DefaultViewMessage::GamePanel(
                    GamePanelMessage::ConfirmUpdate,
                ));

                container(row![].push(play_button).push(update_button).spacing(12))
                    .width(Length::Fill)
                    .align_y(Vertical::Center)
                    .into()
            },
            GamePanelState::Updating { btnstate, .. }
                if *btnstate == DownloadButtonState::InProgress =>
            {
                // When the game is downloading, the download progress bar and related
                // stats replace the Launch / Update button
                let (step, percent, total, downloaded, bytes_per_sec, remaining) =
                    match &self.download_progress {
                        Some(Progress::Incomplete {
                            download,
                            unzip,
                            delete,
                        }) => {
                            let (step, progress) = match (
                                download.is_finished(),
                                unzip.is_finished(),
                                delete.is_finished(),
                            ) {
                                (false, _, _) => ("DOWNLOADING", &download),
                                (true, false, _) => ("UNZIPPING", &unzip),
                                (true, true, false) => ("DELETING", &delete),
                                (true, true, true) => ("FINALIZING", &unzip),
                            };
                            (
                                step,
                                progress.percent_complete() as f32,
                                progress.total_bytes(),
                                progress.processed_bytes(),
                                progress.bytes_per_sec(),
                                progress.time_remaining(),
                            )
                        },
                        Some(Progress::Successful(_)) => {
                            ("FINALIZING", 100.0, 0, 0, 0, Duration::from_secs(0))
                        },
                        _ => ("PREPARING", 0.0, 0, 0, 0, Duration::from_secs(0)),
                    };

                let download_rate = bytes_per_sec as f32 / 1_000_000.0;

                let mut meta_row = row![].spacing(6).align_items(Alignment::Center).push(
                    text(format!(
                        "{} / {}",
                        pretty_bytes(downloaded),
                        pretty_bytes(total)
                    ))
                    .size(12)
                    .style(TextStyle::Secondary),
                );

                if download_rate >= f32::EPSILON {
                    meta_row = meta_row
                        .push(text("·").size(12).style(TextStyle::Muted))
                        .push(
                            text(format!("{download_rate:.1} MB/s"))
                                .font(POPPINS_MEDIUM_FONT)
                                .size(12),
                        )
                        .push(iced::widget::horizontal_space());

                    let seconds = remaining.as_secs() % 60;
                    let minutes = (remaining.as_secs() / 60) % 60;
                    let hours = (remaining.as_secs() / 60) / 60;
                    let remaining_text = if hours > 0 {
                        format!("{hours:02}:{minutes:02}:{seconds:02} left")
                    } else {
                        format!("{minutes:02}:{seconds:02} left")
                    };
                    meta_row = meta_row
                        .push(text(remaining_text).size(12).style(TextStyle::Secondary));
                }

                container(
                    column![]
                        .push(
                            row![]
                                .align_items(Alignment::Center)
                                .push(
                                    text(step)
                                        .font(POPPINS_MEDIUM_FONT)
                                        .size(10)
                                        .style(TextStyle::Muted),
                                )
                                .push(iced::widget::horizontal_space())
                                .push(
                                    text(format!("{percent:.0}%"))
                                        .font(POPPINS_BOLD_FONT)
                                        .size(22),
                                ),
                        )
                        .push(container(text("")).height(Length::Fixed(8.0)))
                        .push(
                            progress_bar(0.0..=100.0f32, percent)
                                .height(Length::Fixed(6.0)),
                        )
                        .push(container(text("")).height(Length::Fixed(10.0)))
                        .push(meta_row)
                        .push(container(text("")).height(Length::Fixed(16.0)))
                        .push(
                            button(
                                text("Cancel")
                                    .font(POPPINS_MEDIUM_FONT)
                                    .size(14)
                                    .horizontal_alignment(Horizontal::Center)
                                    .vertical_alignment(Vertical::Center)
                                    .width(Length::Fill),
                            )
                            .style(ButtonStyle::Danger)
                            .width(Length::Fill)
                            .height(Length::Fixed(36.0))
                            .on_press(
                                DefaultViewMessage::GamePanel(
                                    GamePanelMessage::CancelDownload,
                                ),
                            ),
                        ),
                )
                .into()
            },
            _ => {
                // For all other states, the button is shown with different text/styling
                // dependant on the state
                let (button_text, enabled) = match &self.state {
                    GamePanelState::ReadyToPlay => ("PLAY", true),
                    GamePanelState::Offline(true) => ("PLAY OFFLINE", true),
                    GamePanelState::Offline(false) => ("RETRY", true),
                    GamePanelState::Updating {
                        btnstate: dstate, ..
                    } => match *dstate {
                        DownloadButtonState::Checking => ("CHECKING FOR UPDATES…", false),
                        DownloadButtonState::WaitForConfirm => ("DOWNLOAD", true),
                        _ => unreachable!(),
                    },
                    GamePanelState::Retry => ("RETRY", true),
                    GamePanelState::Playing(_) => ("RUNNING", false),
                    // The "update now?" prompt covers this button while it's up, so
                    // it's just shown disabled underneath -
                    // GamePanelState::UpdateAvailable above is what
                    // actually renders once the user answers.
                    GamePanelState::UpdateAvailable { .. } => unreachable!(),
                    GamePanelState::UpdatePrompt { .. } => ("PLAY", false),
                };

                let mut launch_button = button(
                    text(button_text)
                        .font(POPPINS_BOLD_FONT)
                        .size(26)
                        .horizontal_alignment(Horizontal::Center)
                        .vertical_alignment(Vertical::Center)
                        .width(Length::Fill),
                );

                if let GamePanelState::ReadyToPlay = &self.state
                    && self.selected_server_browser_address.is_some()
                {
                    launch_button = button(
                        column![]
                            .align_items(Alignment::Center)
                            .width(Length::Fill)
                            .padding([10, 40])
                            .push(
                                text("Connect to")
                                    .font(POPPINS_BOLD_FONT)
                                    .line_height(LineHeight::Absolute(22.into()))
                                    .size(18)
                                    .horizontal_alignment(Horizontal::Center)
                                    .vertical_alignment(Vertical::Center),
                            )
                            .push(
                                text("selected server")
                                    .font(POPPINS_BOLD_FONT)
                                    .line_height(LineHeight::Absolute(22.into()))
                                    .size(18)
                                    .horizontal_alignment(Horizontal::Center)
                                    .vertical_alignment(Vertical::Center),
                            ),
                    );
                };

                launch_button = launch_button
                    .style(ButtonStyle::Primary)
                    .width(Length::FillPortion(3))
                    .height(Length::Fixed(72.0));

                if enabled {
                    launch_button = launch_button.on_press(
                        DefaultViewMessage::GamePanel(GamePanelMessage::PlayPressed),
                    );
                }

                let server_browser_button = button(
                    column![]
                        .align_items(Alignment::Center)
                        .width(Length::Fill)
                        .padding([10, 0])
                        .push(
                            text("Server")
                                .font(POPPINS_MEDIUM_FONT)
                                .size(15)
                                .horizontal_alignment(Horizontal::Center)
                                .vertical_alignment(Vertical::Center),
                        )
                        .push(
                            text("Browser")
                                .font(POPPINS_MEDIUM_FONT)
                                .size(15)
                                .horizontal_alignment(Horizontal::Center)
                                .vertical_alignment(Vertical::Center),
                        ),
                )
                .width(Length::FillPortion(1))
                .height(Length::Fixed(72.0))
                .style(ButtonStyle::Secondary)
                .on_press(DefaultViewMessage::Interaction(
                    Interaction::ToggleServerBrowser,
                ));

                container(
                    row![]
                        .push(launch_button)
                        .push(server_browser_button)
                        .spacing(12),
                )
                .width(Length::Fill)
                .align_y(Vertical::Center)
                .into()
            },
        }
    }
}
