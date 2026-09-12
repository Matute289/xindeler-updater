use std::{
    net::{IpAddr, SocketAddr},
    num::NonZeroU32,
    sync::LazyLock,
    time::Duration,
};

use tracing::{debug, warn};
use xindeler_query_server::{
    client::{QueryClient, QueryClientError},
    proto::{ServerIdentity, ServerInfo},
};

static PING_COUNT: LazyLock<NonZeroU32> = LazyLock::new(|| NonZeroU32::new(5).unwrap());

pub async fn create_client(address: &str, port: u16) -> Option<QueryClient> {
    let addr = match address.parse::<IpAddr>() {
        Ok(addr) => SocketAddr::new(addr, port),
        Err(_error) => tokio::net::lookup_host(format!("{address}:{port}"))
            .await
            .inspect_err(|error| warn!(?address, ?error, "Host lookup failed"))
            .ok()?
            .next()
            .or_else(|| {
                warn!(?address, "Host lookup returned no IP addresses");
                None
            })?,
    };

    Some(QueryClient::new(addr))
}

pub async fn perform_ping(
    client: &mut QueryClient,
) -> Result<(Duration, ServerInfo), QueryClientError> {
    let mut avg_ms = 0.0;
    let mut server_info = None;
    let mut last_error = None;

    for attempt in 1..=PING_COUNT.get() {
        match client.server_info().await {
            Ok((new_server_info, ping)) => {
                avg_ms = ping.as_millis() as f32 * (1.0 / attempt as f32)
                    + avg_ms * ((attempt as f32 - 1.0) / attempt as f32);
                server_info = Some(new_server_info);
            },
            Err(error) => {
                last_error = Some(error);
            },
        }
    }

    server_info
        .map(|info| (Duration::from_millis(avg_ms as u64), info))
        .ok_or_else(|| {
            last_error.expect(
                "There must have occurred some error if server_info is not Some()",
            )
        })
}

/// Confirms a server is actually running Xindeler (not vanilla Veloren, nor an
/// unrelated fork) via the `Identity` query. Per the crate's own guidance, any
/// error here (timeout, protocol error, an older server with no match arm for
/// this request) is treated the same as a magic mismatch - "not confirmed", not
/// a separate failure case worth distinguishing to the caller.
pub async fn perform_identity_check(client: &mut QueryClient) -> bool {
    match client.identity().await {
        Ok((identity, _ping)) => identity == ServerIdentity::XINDELER,
        Err(error) => {
            debug!(?error, "Identity check failed or unsupported by server");
            false
        },
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use tokio::sync::watch;
    use xindeler_query_server::{
        proto::ServerBattleMode,
        server::{Metrics, QueryServer},
    };

    use super::*;

    const TEST_SERVER_INFO: ServerInfo = ServerInfo {
        git_hash: 0,
        git_timestamp: 0,
        players_count: 1,
        player_cap: 100,
        battlemode: ServerBattleMode::GlobalPvE,
    };

    /// Exercises this module's own `create_client`/`perform_ping`/
    /// `perform_identity_check` - the exact functions the "add custom server" flow
    /// calls - against a real `QueryServer`, rather than trusting the
    /// xindeler-query-server crate's own examples alone.
    #[tokio::test]
    async fn identity_check_confirms_a_real_xindeler_server() {
        let addr: SocketAddr = "[::1]:24006".parse().unwrap();
        let (_sender, receiver) = watch::channel(TEST_SERVER_INFO);
        let mut server = QueryServer::new(addr, receiver, 10002);
        tokio::task::spawn(async move {
            server.run(Arc::new(Mutex::new(Metrics::default()))).await
        });

        let mut client = create_client("::1", 24006).await.expect("resolve failed");
        let (_ping, info) = perform_ping(&mut client).await.expect("ping failed");
        assert_eq!(info, TEST_SERVER_INFO);
        assert!(perform_identity_check(&mut client).await);
    }
}
