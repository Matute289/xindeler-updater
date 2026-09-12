use crate::{Result, query};
use veloren_serverbrowser_api::GameServerList;

pub(crate) async fn fetch_server_list(url: String) -> Result<GameServerList> {
    let response = query(url).await?;

    let server_list = response.json::<GameServerList>().await?;

    Ok(server_list)
}

#[cfg(test)]
mod tests {
    // Regression coverage for a real bug hit 2026-09-12: xindeler-web-api's
    // /v1/servers endpoint initially serialized an absent location as JSON `null`,
    // but veloren-serverbrowser-api 0.4.0's `GameServer::location` deserializer calls
    // `String::deserialize` directly and has no `null` case, so any server sending an
    // explicit `null` breaks the *entire* list fetch (not just that one field) -
    // surfacing as "Error fetching server list" even though the endpoint is up and
    // returning 200. Omitting the field entirely (what these two cases confirm work)
    // is the only safe way to represent "no location" for this crate version.
    #[test]
    fn omitted_location_field_deserializes() {
        let json = r#"{"servers":[{"name":"Xindeler","address":"server.xindeler.com","port":14004,"description":"The official Xindeler server.","auth_server":"https://auth.xindeler.com","query_port":14006,"channel":"release","official":true}]}"#;
        let parsed: super::GameServerList = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.servers[0].location, None);
    }

    #[test]
    fn valid_country_code_deserializes() {
        let json = r#"{"servers":[{"name":"Xindeler","address":"server.xindeler.com","port":14004,"description":"The official Xindeler server.","location":"US","auth_server":"https://auth.xindeler.com","query_port":14006,"channel":"release","official":true}]}"#;
        let parsed: super::GameServerList = serde_json::from_str(json).unwrap();
        assert!(parsed.servers[0].location.is_some());
    }
}
