//! This module parses command line arguments and returns a parsed struct on which
//! the GUI/CLI can act upon.
use clap::{ArgAction::Count, Parser, Subcommand, crate_authors, crate_version};

/// Provides automatic updates for Xindeler. ( <https://xindeler.com> )
#[derive(Parser, Debug, Default, Clone)]
#[command(name = "XindelerUpdater", version = crate_version!(), author = crate_authors!())]
pub struct CmdLine {
    #[command(subcommand)]
    pub action: Option<Action>,
    /// Set the logging verbosity for Xindeler (v = DEBUG, vv = TRACE)
    #[arg(short, long, action = Count, global = true)]
    pub verbose: u8,
    /// Set the logging verbosity for XindelerUpdater (d = DEBUG, dd = TRACE)
    #[arg(short, long, action = Count, global = true)]
    pub debug: u8,
    /// Force a reset of all user data on startup
    #[arg(long, global = true)]
    pub force_reset: bool,
    /// Skip the real update check and force the game panel into a specific state,
    /// to visually test each case without a real install/server. Debug builds only.
    #[cfg(debug_assertions)]
    #[arg(long, global = true, value_enum)]
    pub mock_state: Option<MockGameState>,
}

/// The `GamePanelState` cases `--mock-state` can force the app to start in. See
/// `GamePanelComponent::mock` for what each one actually sets up.
#[cfg(debug_assertions)]
#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum MockGameState {
    /// "Launch" button - nothing installed/no update pending.
    Ready,
    /// Disabled "Checking..." button, mid update-check.
    Checking,
    /// "Download" button - first install, nothing to keep playing in the meantime
    /// so there's no prompt, straight to the download confirmation.
    WaitForConfirm,
    /// The "update now?" modal on top of an already-installed, playable game.
    UpdatePrompt,
    /// Modal dismissed: Play and Update side by side.
    UpdateAvailable,
    /// "Play Offline" - installed, but the server is unreachable.
    OfflinePlayable,
    /// "Try Again" - never installed, and the server is unreachable.
    OfflineUnplayable,
    /// "Retry" - the last download attempt failed or was cancelled.
    Retry,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Action {
    /// Starts the game without updating.
    Start,
    /// Only updates the game.
    Update,
    /// Update and start the game.
    Run,
    /// Use the CLI to configure profiles.
    Config,
    /// Update the Launcher if possible.
    #[cfg(windows)]
    Upgrade,
}

impl CmdLine {
    /// Parses command line for arguments and returns itself
    pub(crate) fn new() -> Self {
        CmdLine::parse()
    }
}
