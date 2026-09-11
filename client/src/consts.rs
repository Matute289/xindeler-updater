pub const SUPPORTED_SERVER_API_VERSION: u32 = 1;
pub const CACHE_VERSION: u8 = 1;

// Filesystem

#[cfg(windows)]
pub const VOXYGEN_FILE: &str = "xindeler-voxygen.exe";
#[cfg(unix)]
pub const VOXYGEN_FILE: &str = "xindeler-voxygen";

#[cfg(windows)]
pub const LOGS_DIR: &str = "userdata\\voxygen\\logs";

#[cfg(unix)]
pub const LOGS_DIR: &str = "userdata/voxygen/logs";

//#[cfg(windows)]
//pub const SERVER_CLI_FILE: &str = "xindeler-server-cli.exe";
#[cfg(unix)]
pub const SERVER_CLI_FILE: &str = "xindeler-server-cli";

pub const SAVED_STATE_FILE: &str = "xindeler_updater_state.ron";
pub const LOG_FILE: &str = "xindeler-updater.log";

// Networking

// For querying
pub const CHANGELOG_URL: &str =
    "https://raw.githubusercontent.com/Matute289/xindeler-new-horizon/{tag}/CHANGELOG.md";
// {tag} above is filled in with the newest semver tag from this endpoint (falling back to
// the default branch if the lookup fails, e.g. offline). The game repo doesn't publish
// GitHub Releases, only raw tags, so this - not /releases/latest - is the right endpoint.
pub const GAME_REPO_TAGS_URL: &str =
    "https://api.github.com/repos/Matute289/xindeler-new-horizon/tags?per_page=30";
// Used when the tags lookup fails and there's no cached changelog to fall back to.
pub const CHANGELOG_FALLBACK_REF: &str = "development";
// NOTE: nothing has confirmed the xindeler.com paths/subdomains below (news RSS, auth
// server, server list) actually exist yet. Each needs a real, live endpoint (or the
// feature reading it needs to be disabled) before this client is distributed to players.
// For user linking
pub const NEWS_URL: &str = "https://xindeler.com/rss.xml";

// The game's source now lives at Matute289/xindeler-new-horizon on GitHub, not GitLab.
pub const RECENT_CHANGES_URL: &str =
    "https://github.com/Matute289/xindeler-new-horizon/pulls?q=is%3Apr+is%3Amerged+sort%3Aupdated-desc";

pub const XINDELER_UPDATER_RELEASE_URL: &str =
    "https://github.com/Matute289/xindeler-updater/releases";

// The launcher's own release manifest (published by .github/workflows/build.yml, see
// NH-145) - same {version, platforms:[{os,arch,file,size,sha256,signed}]} shape as the
// game's own latest.json. Used by launcher_update.rs to check for and apply updates to
// xindeler-updater itself, without depending on the GitHub Releases API.
pub const UPDATER_MANIFEST_URL: &str =
    "https://downloads.xindeler.com/updater-releases/updater-latest.json";
// Base path new versions get published under - <this>/<version>/<file>.
pub const UPDATER_RELEASES_BASE_URL: &str =
    "https://downloads.xindeler.com/updater-releases";

pub const OFFICIAL_AUTH_SERVER: &str = "https://auth.xindeler.com";

pub const OFFICIAL_SERVER_LIST: &str = "https://serverlist.xindeler.com";

// NOTE: placeholder until there's a real "get your server listed" doc page (xindeler-wiki
// or xindeler-documentation) to point to — an issue against the game repo is at least a
// real, working destination today.
pub const SERVER_LISTING_REQUEST_URL: &str =
    "https://github.com/Matute289/xindeler-new-horizon/issues/new";
