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
// NOTE: the URLs below still point at Veloren's own live services (news RSS, community showcase,
// merged-MR feed, auth server, server list, server-browser inclusion doc) — Xindeler does not yet
// run equivalents for any of these, so these values are placeholders kept only so the crate still
// compiles; each needs a real Xindeler URL, or the feature reading it needs to be disabled, before
// this client is distributed to players.
// For user linking
pub const NEWS_URL: &str = "https://veloren.net/rss.xml";

pub const COMMUNITY_SHOWCASE_URL: &str = "https://veloren.net/community-showcase/rss.xml";

pub const GITLAB_MERGED_MR_URL: &str =
    "https://gitlab.com/veloren/veloren/-/merge_requests?scope=all&sort=merged_at_desc&state=merged";

pub const XINDELER_UPDATER_RELEASE_URL: &str =
    "https://github.com/Matute289/xindeler-updater/releases";

pub const OFFICIAL_AUTH_SERVER: &str = "https://auth.veloren.net";

pub const OFFICIAL_SERVER_LIST: &str = "https://serverlist.veloren.net";

pub const GITLAB_SERVER_BROWSER_URL: &str =
    "https://gitlab.com/veloren/serverbrowser#inclusion-of-new-servers-to-the-list";
