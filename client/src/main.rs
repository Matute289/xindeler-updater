mod assets;
mod channels;
mod cli;
mod consts;
mod error;
mod gui;
mod io;
mod launcher_update;
mod logger;
mod net;
#[cfg(unix)]
mod nix;
mod profiles;
mod update;
#[cfg(windows)]
mod windows;

// Loads client/locales/*.yml at compile time. English is the fallback for any key a
// translation is missing, so a half-translated locale degrades to English rather than
// rendering a raw key. The active locale is set from `Profile::language` in
// `Profile::load()` and switched live from the settings panel.
rust_i18n::i18n!("locales", fallback = "en");

use crate::error::ClientError;

pub use io::*;
pub use net::*;

pub type Result<T> = std::result::Result<T, ClientError>;

fn main() {
    error::panic_hook();

    // If we fail to read a line, the user probably cancelled an action
    if let Some(e) = cli::process()
        .err()
        .filter(|e| !matches!(e, ClientError::Readline(_)))
    {
        tracing::error!("{}", e);
        tracing::info!("Press enter to exit...");
        let _ = std::io::stdin().read_line(&mut String::new());
    }
}
