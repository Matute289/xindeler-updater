use std::net::IpAddr;

use country_parser::Country;
use tracing::debug;

/// Looks up the ISO 3166-1 alpha-2 country an IP is in, via a free, no-API-key
/// geolocation service (`ipwho.is`). Country-level accuracy from IP geolocation is
/// reliable (IP blocks are allocated per-country); city-level would not be, which is
/// why this only asks for `country_code` - it's also all `GameServer::location`
/// actually holds.
///
/// Only ever called once, when a custom server is added or edited in the server
/// browser (see `server_browser_panel.rs`'s `AddCustomServerSubmit`) - matches Matías's
/// request to fill the field in at registration time, not on every ping/refresh.
pub async fn lookup_country(ip: IpAddr) -> Option<Country> {
    #[derive(serde::Deserialize)]
    struct GeoResponse {
        success: bool,
        country_code: Option<String>,
    }

    let url = format!("https://ipwho.is/{ip}?fields=success,country_code");
    let response = match crate::query(url).await {
        Ok(response) => response,
        Err(error) => {
            debug!(?ip, ?error, "Geolocation lookup request failed");
            return None;
        },
    };

    let parsed = match response.json::<GeoResponse>().await {
        Ok(parsed) => parsed,
        Err(error) => {
            debug!(?ip, ?error, "Geolocation lookup response wasn't valid JSON");
            return None;
        },
    };

    if !parsed.success {
        debug!(?ip, "Geolocation lookup reported failure for this IP");
        return None;
    }

    parsed.country_code.and_then(country_parser::parse)
}
