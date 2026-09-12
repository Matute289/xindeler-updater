use crate::Result;
use reqwest::IntoUrl;

// Name your user agent after your app?
const USER_AGENT: &str = concat!("XindelerUpdater/", env!("CARGO_PKG_VERSION"));

lazy_static::lazy_static! {
    // Base for config, profiles, ...
    pub static ref WEB_CLIENT: reqwest::Client = {
        reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .use_rustls_tls()
            .connect_timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("FATAL: Failed to build reqwest client!")
    };

    pub static ref GITHUB_CLIENT: reqwest::Client = {
        reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .http2_prior_knowledge()
            .use_rustls_tls()
            .connect_timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("FATAL: Failed to build reqwest client!")
    };
}

/// Queries url for the etag header
pub(crate) async fn query_etag<U: IntoUrl>(url: U) -> Result<Option<String>> {
    Ok(WEB_CLIENT
        .head(url)
        .send()
        .await?
        .headers()
        .get("etag")
        .and_then(|s| s.to_str().map(String::from).ok()))
}

/// Extracts Etag value from response
/// Note: Will default to `MISSING_ETAG` incase header isn't found
pub(crate) fn get_etag(x: &reqwest::Response) -> String {
    x.headers().get("etag").map(|x| x.to_str().unwrap().to_string()) // Etag will always be a valid UTF-8 due to it being ASCII
        .unwrap_or_else(|| "MISSING_ETAG".into())
}

/// Errors on a non-2xx response instead of returning it as `Ok`. Without this, a
/// transient 5xx (found in the wild: raw.githubusercontent.com serving a Varnish
/// "503 Backend.max_conn reached" HTML page) gets read as if it were the real
/// response body by every caller - e.g. the changelog panel would markdown-parse the
/// error page, find no `##` headings, and silently cache an empty changelog as if it
/// had loaded successfully, rather than the fetch failing so its one retry can kick
/// in.
pub(crate) async fn query<U: IntoUrl>(url: U) -> Result<reqwest::Response> {
    Ok(WEB_CLIENT.get(url).send().await?.error_for_status()?)
}

#[derive(serde::Deserialize)]
struct GithubTag {
    name: String,
}

/// Returns the newest semver tag (e.g. "v0.25.4") in the game repo, per
/// `consts::GAME_REPO_TAGS_URL`. The repo only publishes raw git tags, not GitHub
/// Releases, so this can't use the simpler `/releases/latest` endpoint - it fetches the
/// tag list and picks the highest semver itself, since GitHub doesn't guarantee the tags
/// endpoint is chronologically ordered.
pub(crate) async fn fetch_latest_game_tag() -> Result<String> {
    let tags: Vec<GithubTag> = GITHUB_CLIENT
        .get(crate::consts::GAME_REPO_TAGS_URL)
        .send()
        .await?
        .json()
        .await?;

    tags.into_iter()
        .filter_map(|tag| {
            let version =
                semver::Version::parse(tag.name.trim_start_matches('v')).ok()?;
            Some((version, tag.name))
        })
        .max_by(|(a, _), (b, _)| a.cmp(b))
        .map(|(_, name)| name)
        .ok_or_else(|| "No semver-tagged releases found".to_string().into())
}
