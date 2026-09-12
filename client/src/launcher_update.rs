//! Cross-platform self-update for the launcher binary itself (not the game - see
//! `update.rs` for that).
//!
//! Sources version/download/checksum info from our own release manifest
//! (`consts::UPDATER_MANIFEST_URL`, published by `.github/workflows/build.yml` for
//! every tagged release of this repo) rather than the GitHub Releases API, so it's the
//! same source of truth xindeler-web-api and xindeler-new-horizon already use, and lets
//! us verify a sha256 before ever executing anything we downloaded.
//!
//! Windows ships an NSIS installer (not a portable binary), so it's applied by
//! downloading and running the installer elevated (see `windows::run_installer`).
//! macOS and Linux ship a portable archived binary, so it's applied by extracting the
//! archive and replacing the running executable in place (`self_update::self_replace`),
//! then relaunching a fresh process.

use crate::{Result, WEB_CLIENT, consts, error::ClientError};
use semver::Version;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
struct UpdaterManifest {
    version: String,
    platforms: Vec<UpdaterManifestPlatform>,
}

#[derive(Debug, Clone, Deserialize)]
struct UpdaterManifestPlatform {
    os: String,
    arch: String,
    file: String,
    sha256: String,
}

/// A newer launcher version found on `consts::UPDATER_MANIFEST_URL` for the running
/// OS/arch.
#[derive(Debug, Clone)]
pub struct LauncherUpdate {
    pub version: String,
    download_url: String,
    sha256: String,
    file_name: String,
}

/// Checks the launcher's own manifest once. `None` if we're already on the latest
/// version, there's no published entry for this OS/arch, or the check fails for any
/// reason (e.g. offline) - a failed check should never block the user from playing.
pub async fn check_for_update() -> Option<LauncherUpdate> {
    let manifest: UpdaterManifest = WEB_CLIENT
        .get(consts::UPDATER_MANIFEST_URL)
        .send()
        .await
        .inspect_err(|e| tracing::debug!(?e, "Launcher update check failed"))
        .ok()?
        .json()
        .await
        .ok()?;

    let remote = Version::parse(manifest.version.trim_start_matches('v')).ok()?;
    let current = Version::parse(env!("CARGO_PKG_VERSION")).ok()?;
    if remote <= current {
        return None;
    }

    let entry = manifest
        .platforms
        .into_iter()
        .find(|p| p.os == std::env::consts::OS && p.arch == std::env::consts::ARCH)?;

    Some(LauncherUpdate {
        download_url: format!(
            "{}/{}/{}",
            consts::UPDATER_RELEASES_BASE_URL,
            manifest.version,
            entry.file
        ),
        sha256: entry.sha256,
        file_name: entry.file,
        version: manifest.version,
    })
}

fn update_cache_dir() -> PathBuf {
    crate::fs::get_cache_path().join("launcher-update")
}

async fn download_and_verify(update: &LauncherUpdate) -> Result<PathBuf> {
    let cache_dir = update_cache_dir();
    let _ = tokio::fs::remove_dir_all(&cache_dir).await;
    tokio::fs::create_dir_all(&cache_dir).await?;

    tracing::info!(url = %update.download_url, "Downloading launcher update");
    let bytes = WEB_CLIENT
        .get(&update.download_url)
        .send()
        .await?
        .bytes()
        .await?;

    let actual_sha256 = hex::encode(Sha256::digest(&bytes));
    if actual_sha256 != update.sha256 {
        return Err(ClientError::Custom(format!(
            "Downloaded launcher update failed checksum verification (expected {}, got \
             {actual_sha256}) - refusing to run it",
            update.sha256
        )));
    }

    let download_path = cache_dir.join(&update.file_name);
    tokio::fs::write(&download_path, &bytes).await?;
    Ok(download_path)
}

/// Downloads, verifies, and applies a launcher update. On success this does not
/// return - the process either relaunches itself (macOS/Linux) or hands off to an
/// elevated installer (Windows) and exits. Only returns on failure, so the caller can
/// show an error and let the user retry.
pub async fn apply(
    update: LauncherUpdate,
    profile: crate::profiles::Profile,
) -> Result<()> {
    let downloaded = download_and_verify(&update).await?;

    // Persist a marker BEFORE replacing/exiting, so the new process can show a
    // one-time "updated successfully" notice on its very next startup - this process
    // is about to exit and can't show anything itself. Best-effort: if this write
    // fails we just silently skip the notice, not fatal to the actual update.
    let mut marked_profile = profile;
    marked_profile.pending_launcher_update_notice =
        Some(env!("CARGO_PKG_VERSION").to_owned());
    if let Err(e) = marked_profile.save().await {
        tracing::warn!(?e, "Failed to persist the launcher-update-succeeded notice");
    }

    #[cfg(windows)]
    {
        crate::windows::run_installer_elevated(&downloaded)?;
        std::process::exit(0);
    }

    #[cfg(unix)]
    {
        unix::replace_and_relaunch(&downloaded)?;
        std::process::exit(0);
    }
}

#[cfg(unix)]
mod unix {
    use crate::{Result, error::ClientError};
    use std::path::{Path, PathBuf};

    const BIN_NAME_IN_ARCHIVE: &str = "xindeler-updater";

    /// Extracts the portable binary out of the downloaded archive (zip on macOS,
    /// tar.gz on Linux - see `.github/workflows/build.yml`'s `Package` steps) into
    /// `extract_dir`, marks it executable, and returns its path. Split out from
    /// `replace_and_relaunch` so it's testable without touching the real running
    /// process.
    fn extract_binary(archive_path: &Path, extract_dir: &Path) -> Result<PathBuf> {
        let _ = std::fs::remove_dir_all(extract_dir);
        std::fs::create_dir_all(extract_dir)?;

        #[cfg(target_os = "macos")]
        self_update::Extract::from_source(archive_path)
            .archive(self_update::ArchiveKind::Zip)
            .extract_file(extract_dir, BIN_NAME_IN_ARCHIVE)?;
        #[cfg(target_os = "linux")]
        self_update::Extract::from_source(archive_path)
            .archive(self_update::ArchiveKind::Tar(Some(
                self_update::Compression::Gz,
            )))
            .extract_file(extract_dir, BIN_NAME_IN_ARCHIVE)?;

        let new_exe = extract_dir.join(BIN_NAME_IN_ARCHIVE);

        let mut perms = std::fs::metadata(&new_exe)?.permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut perms, 0o755);
        std::fs::set_permissions(&new_exe, perms)?;

        Ok(new_exe)
    }

    /// Extracts the portable binary from the downloaded archive, replaces the
    /// currently-running executable with it, and relaunches.
    pub(super) fn replace_and_relaunch(archive_path: &Path) -> Result<()> {
        let extract_dir = archive_path
            .parent()
            .ok_or_else(|| ClientError::Custom("bad archive path".to_owned()))?
            .join("extracted");
        let new_exe = extract_binary(archive_path, &extract_dir)?;

        self_update::self_replace::self_replace(&new_exe).map_err(|e| {
            ClientError::Custom(format!("Failed to replace launcher: {e}"))
        })?;

        let current_exe = std::env::current_exe()?;
        std::process::Command::new(current_exe).spawn()?;

        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use super::extract_binary;
        use std::os::unix::fs::PermissionsExt;

        /// Proves `self_update::Extract`'s API is being called correctly for this
        /// platform's archive format - the riskiest part of this module to get wrong
        /// by inspection alone, since a mistake here would only surface the first
        /// time a real user's launcher tries to auto-update.
        #[test]
        fn extracts_the_binary_out_of_a_real_archive() {
            let tmp = tempfile::tempdir().expect("tempdir");
            let original_content = b"pretend this is the xindeler-updater binary";
            let bin_path = tmp.path().join("xindeler-updater");
            std::fs::write(&bin_path, original_content).unwrap();

            #[cfg(target_os = "macos")]
            let archive_path = {
                // The real `zip` CLI silently appends ".zip" to the output name if
                // it's missing - matching that here, rather than assuming, is the
                // whole point of driving this through the actual CLI instead of a
                // Rust zip-writing crate.
                let path = tmp.path().join("archive");
                let status = std::process::Command::new("zip")
                    .args(["-j"])
                    .arg(&path)
                    .arg(&bin_path)
                    .status()
                    .expect("run zip");
                assert!(status.success());
                path.with_extension("zip")
            };
            #[cfg(target_os = "linux")]
            let archive_path = {
                let path = tmp.path().join("archive.tar.gz");
                let status = std::process::Command::new("tar")
                    .arg("-czf")
                    .arg(&path)
                    .args(["-C"])
                    .arg(tmp.path())
                    .arg("xindeler-updater")
                    .status()
                    .expect("run tar");
                assert!(status.success());
                path
            };

            let extract_dir = tmp.path().join("extracted");
            let extracted = extract_binary(&archive_path, &extract_dir)
                .expect("extraction should succeed");

            assert_eq!(std::fs::read(&extracted).unwrap(), original_content);
            let mode = std::fs::metadata(&extracted).unwrap().permissions().mode();
            assert_eq!(mode & 0o111, 0o111, "extracted binary should be executable");
        }
    }
}
