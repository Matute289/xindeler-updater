pub mod client;
pub mod ping;
pub mod server_list;

pub use client::*;

pub const DEFAULT_GAME_PORT: u16 = 14004;
// Default port used for server information queries when the caller doesn't know a
// server's actual query port - matches `GameServer::query_port`'s doc comment.
pub const DEFAULT_QUERY_PORT: u16 = 14006;
