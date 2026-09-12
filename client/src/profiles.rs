use crate::{Result, channels::Channel, consts, fs};
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    ffi::OsString,
    path::{Path, PathBuf},
    process::Stdio,
};
use tokio::{fs::File, io::AsyncWriteExt, process::Command};
use tracing::{error, warn};

// TODO: Support multiple profiles and manage them here.

/// Represents a version with channel, name and path.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub server: Server,
    pub channel: Channel,
    // FIXME: This field is currently ignored.
    // Persisting the storage path led to problems with the snap package because
    // the directory the snap is allowed to write to changes with each new snap version.
    // Since there is currently no use in persisting this path anyway, we ignore it.
    // It is not removed either to guarantee backwards-compatibility by making sure
    // configuration files containing this field can still be successfully parsed
    #[serde(rename = "directory")]
    _directory: PathBuf,
    pub version: Option<String>,
    pub wgpu_backend: WgpuBackend,
    pub log_level: LogLevel,
    pub env_vars: String,
    // TODO: make a file-picker UI for this
    pub assets_override: Option<String>,
    pub wgpu_device: WgpuDevice,

    /// used to avoid duplicate redownload of patched binaries on nixos
    pub patched_crc32s: Vec<PatchedInfo>,

    /// If true, a new game version is downloaded/installed automatically as soon as
    /// it's found, with no confirmation prompt - just a one-time "updated
    /// successfully" notice once it's done. Independent of `auto_update_launcher`.
    /// `#[serde(default)]` so profiles saved before this field existed still load
    /// (defaulting to false, the safer opt-in behavior).
    #[serde(default)]
    pub auto_update_game: bool,
    /// Same as `auto_update_game`, but for the launcher itself.
    #[serde(default)]
    pub auto_update_launcher: bool,
    /// Set right before an auto-applied launcher update replaces/relaunches the
    /// process, so the new process can show a one-time "updated successfully" notice
    /// on its next startup instead of silently going quiet.
    #[serde(default)]
    pub pending_launcher_update_notice: Option<String>,
    /// Language the launcher UI is rendered in. `#[serde(default)]` for the same
    /// reason as the two fields above: profiles saved before this field existed must
    /// still load, falling back to `Language::English`.
    #[serde(default)]
    pub language: Language,
    /// Servers the player added manually by IP/DNS name, in addition to the curated
    /// list fetched from `OFFICIAL_SERVER_LIST`. `#[serde(default)]` for the same
    /// backward-compat reason as the fields above.
    #[serde(default)]
    pub custom_servers: Vec<veloren_serverbrowser_api::GameServer>,

    #[serde(skip)]
    pub supported_wgpu_backends: Vec<WgpuBackend>,
    #[serde(skip)]
    pub supported_wgpu_devices: Vec<WgpuDevice>,
}

const DEFAULT_PROFILE_NAME: &str = "default";
impl Default for Profile {
    fn default() -> Self {
        Profile::new(
            DEFAULT_PROFILE_NAME.to_owned(),
            Server::Production,
            Channel("release".to_owned()),
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchedInfo {
    pub(crate) local_unix_path: String,
    pub(crate) pre_crc32: u32,
    pub(crate) post_crc32: u32,
}

#[derive(
    Debug, derive_more::Display, Clone, Copy, Serialize, Deserialize, PartialEq, Eq,
)]
pub enum WgpuBackend {
    Auto,
    OpenGl,
    DX11,
    DX12,
    Metal,
    Vulkan,
}

// Kept in sync with the game's own hardcoded list (xindeler-new-horizon,
// voxygen/src/main.rs's ListWgpuBackends) - it doesn't query anything either, and this
// is only the pre-install fallback shown before the launcher can ask the real game
// binary. DX11 is deliberately absent: wgpu dropped that backend in 0.19, and the game
// doesn't parse "dx11" either (falls through to its own default) - so it was a
// selectable option here that silently did nothing.
#[cfg(target_os = "windows")]
static WGPU_BACKENDS: &[WgpuBackend] = &[
    WgpuBackend::Auto,
    WgpuBackend::OpenGl,
    WgpuBackend::DX12,
    WgpuBackend::Vulkan,
];

#[cfg(target_os = "linux")]
static WGPU_BACKENDS: &[WgpuBackend] =
    &[WgpuBackend::Auto, WgpuBackend::OpenGl, WgpuBackend::Vulkan];

#[cfg(target_os = "macos")]
static WGPU_BACKENDS: &[WgpuBackend] = &[WgpuBackend::Auto, WgpuBackend::Metal];

#[derive(Debug, derive_more::Display, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum WgpuDevice {
    Auto,
    Manual(String),
}

pub async fn query_wgpu_backends(process_path: &Path) -> Vec<WgpuBackend> {
    if let Some(res) = Command::new(process_path)
        .arg("list-wgpu-backends")
        .stdout(Stdio::piped())
        .output()
        .await
        .ok()
        .filter(|res| res.status.success())
    {
        let res = String::from_utf8_lossy(&res.stdout);
        res.lines()
            .filter_map(|backend| {
                Some(match backend {
                    "vulkan" => WgpuBackend::Vulkan,
                    "dx11" => WgpuBackend::DX11,
                    "dx12" => WgpuBackend::DX12,
                    "opengl" => WgpuBackend::OpenGl,
                    "metal" => WgpuBackend::Metal,
                    other => {
                        error!(?other, "Invalid list-wgpu-backends output detected");
                        return None;
                    },
                })
            })
            .chain(std::iter::once(WgpuBackend::Auto))
            .collect()
    } else {
        error!("failed to query WGPU Backends, falling back to defaults");
        WGPU_BACKENDS.to_vec()
    }
}

pub async fn query_wgpu_devices(process_path: &Path) -> Vec<WgpuDevice> {
    if let Some(res) = Command::new(process_path)
        .arg("list-wgpu-devices")
        .stdout(Stdio::piped())
        .output()
        .await
        .ok()
        .filter(|res| res.status.success())
    {
        let res = String::from_utf8_lossy(&res.stdout);

        res.lines()
            .map(|device| WgpuDevice::Manual(device.to_string()))
            .chain(std::iter::once(WgpuDevice::Auto))
            .collect()
    } else {
        warn!("Failed to query WGPU Devices, falling back to defaults");
        vec![WgpuDevice::Auto]
    }
}

#[derive(
    Debug, derive_more::Display, Clone, Copy, Serialize, Deserialize, PartialEq, Eq,
)]
pub enum Server {
    Production,
}

pub static SERVERS: &[Server] = &[Server::Production];

#[derive(
    Debug,
    Default,
    derive_more::Display,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
)]
pub enum LogLevel {
    #[default]
    Default,
    Debug,
    Trace,
}

pub static LOG_LEVELS: &[LogLevel] =
    &[LogLevel::Default, LogLevel::Debug, LogLevel::Trace];

/// The language the launcher's own UI is rendered in. Only affects XindelerUpdater -
/// the game reads its own language setting from its own config.
// `Default` is derived rather than hand-written (clippy::derivable_impls), matching
// how `LogLevel` above declares its own default variant.
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Language {
    #[default]
    English,
    EsLatam,
}

// Deliberately not `derive_more::Display`: that would render the bare variant ident
// ("EsLatam") in the settings dropdown. These are the endonyms shown to the user.
impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Language::English => "English",
            Language::EsLatam => "Español (Latinoamérica)",
        })
    }
}

impl Language {
    /// The locale code handed to `rust_i18n::set_locale`, matching the file names in
    /// `client/locales/`.
    pub fn code(&self) -> &'static str {
        match self {
            Language::English => "en",
            Language::EsLatam => "es",
        }
    }
}

pub static LANGUAGES: &[Language] = &[Language::English, Language::EsLatam];

impl Server {
    pub fn url(&self) -> &str {
        match self {
            Server::Production => "https://downloads.xindeler.com",
        }
    }
}

impl Profile {
    pub fn new(name: String, server: Server, channel: Channel) -> Self {
        Self {
            _directory: fs::profile_path(&name),
            name,
            server,
            channel,
            version: None,
            wgpu_backend: WgpuBackend::Auto,
            log_level: LogLevel::Default,
            env_vars: String::new(),
            assets_override: None,
            patched_crc32s: Vec::new(),
            auto_update_game: false,
            auto_update_launcher: false,
            pending_launcher_update_notice: None,
            language: Language::default(),
            custom_servers: Vec::new(),
            supported_wgpu_backends: Vec::new(),
            wgpu_device: WgpuDevice::Auto,
            supported_wgpu_devices: Vec::new(),
        }
    }

    pub fn load() -> Self {
        fs::verify_cache();
        let saved_state_file = fs::savedstate_file();
        let profile = match std::fs::File::open(&saved_state_file) {
            Ok(file) => {
                match ron::de::from_reader(file) {
                    Ok(profile) => {
                        // Rust type inference magic
                        let mut profile: Profile = profile;
                        profile.reload_wgpu_backends();
                        profile.reload_wgpu_devices();
                        profile
                    },
                    Err(e) => {
                        tracing::debug!(
                            "Decoding state failed. Falling back to default: {}",
                            e
                        );
                        Self::default()
                    },
                }
            },
            Err(e) => {
                tracing::debug!(
                    ?e,
                    "Failed to read saved state from {}, falling back to default state",
                    saved_state_file.to_string_lossy()
                );
                Self::default()
            },
        };

        // Applied here rather than in the GUI so the very first `view()` already
        // renders in the persisted language - no English flash before the user's
        // choice takes effect.
        rust_i18n::set_locale(profile.language.code());

        profile
    }

    pub async fn save(self) -> Result<()> {
        let data = tokio::task::block_in_place(|| {
            ron::ser::to_string_pretty(&self, PrettyConfig::default())
        })?;
        let mut file = File::create(fs::savedstate_file()).await?;
        file.write_all(data.as_bytes()).await?;
        file.sync_all().await?;

        Ok(())
    }

    pub async fn save_ref(&self) -> Result<()> {
        let data = tokio::task::block_in_place(|| {
            ron::ser::to_string_pretty(self, PrettyConfig::default())
        })?;
        let mut file = File::create(fs::savedstate_file()).await?;
        file.write_all(data.as_bytes()).await?;
        file.sync_all().await?;

        Ok(())
    }

    pub fn directory(&self) -> PathBuf {
        fs::profile_path(&self.name)
    }

    /// Returns path to voxygen binary.
    /// e.g. <base>/profiles/default/xindeler-voxygen.exe
    pub fn voxygen_path(&self) -> PathBuf {
        self.directory().join(consts::VOXYGEN_FILE)
    }

    /// Returns path to the voxygen logs directory
    /// e.g. <base>/profiles/default/logs
    pub fn voxygen_logs_path(&self) -> PathBuf {
        self.directory().join(consts::LOGS_DIR)
    }

    /// Returns the download url for this profile
    pub fn download_url(&self) -> String {
        format!(
            "{}/updater/latest/{}/{}/{}",
            self.server.url(),
            std::env::consts::OS,
            std::env::consts::ARCH,
            // Channel::Display capitalizes the first letter for UI presentation
            // ("release" -> "Release" in the settings dropdown) - the server only
            // publishes the raw lowercase channel name as a URL path segment, so this
            // must bypass Display and use the underlying string directly.
            self.channel.0
        )
    }

    pub(crate) fn version_url(&self) -> String {
        format!(
            "{}/updater/version/{}/{}/{}",
            self.server.url(),
            std::env::consts::OS,
            std::env::consts::ARCH,
            self.channel.0
        )
    }

    pub(crate) fn channel_url(&self) -> String {
        format!(
            "{}/updater/channels/{}/{}",
            self.server.url(),
            std::env::consts::OS,
            std::env::consts::ARCH,
        )
    }

    pub(crate) fn api_version_url(&self) -> String {
        format!("{}/updater/api/version", self.server.url(),)
    }

    pub(crate) fn announcement_url(&self) -> String {
        format!("{}/announcement", self.server.url(),)
    }

    // TODO: add possibility to start the server too
    pub fn start(profile: &Profile, game_server_address: Option<&str>) -> Command {
        let mut envs = HashMap::new();
        let userdata_dir = profile.directory().join("userdata").into_os_string();
        let screenshot_dir = profile.directory().join("screenshots").into_os_string();
        let assets_dir = profile.directory().join("assets").into_os_string();

        if profile.log_level != LogLevel::Default {
            let log_level = match profile.log_level {
                LogLevel::Default => OsString::from("info"),
                LogLevel::Debug => OsString::from("debug"),
                LogLevel::Trace => OsString::from("trace"),
            };
            envs.insert("RUST_LOG", log_level);
        }

        if let Some(path) = &profile.assets_override {
            if Path::new(&path).is_dir() {
                envs.insert("VELOREN_ASSETS_OVERRIDE", path.into());
            } else {
                tracing::warn!(
                    "We can't find this assets override as a directory: {}",
                    path
                );
            }
        }

        envs.insert("VOXYGEN_SCREENSHOT", screenshot_dir);
        envs.insert("VELOREN_USERDATA", userdata_dir);
        envs.insert("VELOREN_ASSETS", assets_dir);

        if profile.wgpu_backend != WgpuBackend::Auto {
            let wgpu_backend = match profile.wgpu_backend {
                WgpuBackend::OpenGl => "gl",
                WgpuBackend::DX11 => "dx11",
                WgpuBackend::DX12 => "dx12",
                WgpuBackend::Metal => "metal",
                WgpuBackend::Vulkan => "vulkan",
                _ => unreachable!(
                    "Unsupported WgpuBackend value: {}",
                    profile.wgpu_backend
                ),
            };
            envs.insert("WGPU_BACKEND", OsString::from(wgpu_backend));
        }

        if profile.wgpu_device != WgpuDevice::Auto {
            envs.insert(
                "WGPU_ADAPTER",
                OsString::from(&profile.wgpu_device.to_string()),
            );
        }

        let (env_vars, env_var_errors) = parse_env_vars(&profile.env_vars);
        for err in env_var_errors {
            tracing::warn!("Environment variable error: {}", err);
        }
        for (var, value) in env_vars {
            envs.insert(var, OsString::from(value));
        }

        tracing::debug!("Launching {}", profile.voxygen_path().display());
        tracing::debug!("CWD: {:?}", profile.directory());
        tracing::debug!("ENV: {:?}", envs);

        let mut cmd = Command::new(profile.voxygen_path());
        cmd.current_dir(profile.directory());
        cmd.envs(envs);

        // If a server is selected in the server browser pass it through to Voxygen
        if let Some(game_server_address) = game_server_address {
            cmd.args(["--server", game_server_address]);
        }

        cmd
    }

    /// Returns whether the profile is ready to be started
    pub fn installed(&self) -> bool {
        self.voxygen_path().exists() && self.version.is_some()
    }

    pub fn reload_wgpu_backends(&mut self) {
        if self.installed() {
            self.supported_wgpu_backends = iced::futures::executor::block_on(
                query_wgpu_backends(&self.voxygen_path()),
            );
            let supported = |backend| self.supported_wgpu_backends.contains(&backend);
            // Update selected backend if it isn't available.
            if self.wgpu_backend != WgpuBackend::Auto && !supported(self.wgpu_backend) {
                self.wgpu_backend = match self.wgpu_backend {
                    WgpuBackend::DX11 if supported(WgpuBackend::OpenGl) => {
                        WgpuBackend::OpenGl
                    },
                    WgpuBackend::OpenGl if supported(WgpuBackend::DX11) => {
                        WgpuBackend::DX11
                    },
                    _ => WgpuBackend::Auto,
                };
            }
        } else {
            // Nothing installed yet to actually ask, so there's no real list to
            // query - but the Settings dropdown still needs *something* selectable
            // (previously this was an empty Vec, leaving the dropdown with nothing
            // to pick even though it displays "Auto" as the current value). Same
            // static per-OS fallback query_wgpu_backends() itself falls back to when
            // the game binary exists but fails to answer.
            self.supported_wgpu_backends = WGPU_BACKENDS.to_vec();
        }
    }

    pub fn reload_wgpu_devices(&mut self) {
        if self.installed() {
            self.supported_wgpu_devices = iced::futures::executor::block_on(
                query_wgpu_devices(&self.voxygen_path()),
            );

            if self.wgpu_device != WgpuDevice::Auto
                && !self.supported_wgpu_devices.contains(&self.wgpu_device)
            {
                self.wgpu_device = WgpuDevice::Auto
            }
        } else {
            self.supported_wgpu_devices = vec![WgpuDevice::Auto];
        }
    }
}

pub fn parse_env_vars(env_vars: &str) -> (Vec<(&str, &str)>, Vec<String>) {
    let env_vars = env_vars.trim();
    let mut errors = Vec::new();

    let vars = if env_vars.is_empty() {
        Vec::new()
    } else {
        env_vars
            .split(',')
            .filter_map(|var| {
                let var = var.trim();
                if let Some((key, value)) = var.split_once('=') {
                    let key = key.trim();
                    let value = value.trim();
                    if key.chars().count() == 0 {
                        errors.push(format!("Invalid variable '{}'", key))
                    }
                    Some((key, value))
                } else {
                    if var.chars().count() == 0 {
                        errors.push("Unnecessary ',' in variable list".to_string());
                    } else {
                        errors.push(format!(
                            "Variable '{}' has no corresponding value",
                            var
                        ));
                    }
                    None
                }
            })
            .collect()
    };
    (vars, errors)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_config() {
        let (vars, errors) = parse_env_vars("");
        assert_eq!(vars, Vec::new());
        assert!(errors.is_empty());
    }

    #[test]
    fn test_normal_config() {
        let (vars, errors) =
            parse_env_vars("       FOO=foo, BAR= bar,BAZ = baz, BAK = bak  ");
        assert_eq!(vars, vec![
            ("FOO", "foo"),
            ("BAR", "bar"),
            ("BAZ", "baz"),
            ("BAK", "bak")
        ]);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_bad_config() {
        let (vars, errors) =
            parse_env_vars("      FOO=foo,,BAR= bar,    = baz, BAK =   , EMM ");
        assert_eq!(vars, vec![
            ("FOO", "foo"),
            ("BAR", "bar"),
            ("", "baz"),
            ("BAK", "")
        ]);
        assert_eq!(errors, vec![
            "Unnecessary ',' in variable list".to_string(),
            "Invalid variable ''".to_string(),
            "Variable 'EMM' has no corresponding value".to_string()
        ]);
    }
}
