use crate::{
    Result,
    assets::{
        GLOBE_ICON, KEY_ICON, PENCIL_ICON, PING_ERROR_ICON, PING_NONE_ICON, PING1_ICON,
        PING2_ICON, PING3_ICON, PING4_ICON, POPPINS_MEDIUM_FONT, STAR_ICON, TRASH_ICON,
        UNIVERSAL_FONT, UP_RIGHT_ARROW_ICON,
    },
    consts,
    consts::{OFFICIAL_SERVER_LIST, SERVER_LISTING_REQUEST_URL},
    gui::{
        components::GamePanelMessage,
        custom_widgets::modal_shell,
        style::{
            ARCANE_500,
            button::{BrowserButtonStyle, ButtonStyle, ServerListEntryButtonState},
            container::ContainerStyle,
            text::TextStyle,
        },
        views::{
            Action,
            default::{DefaultViewMessage, Interaction},
        },
        widget::*,
    },
    net,
    profiles::Profile,
    server_list::fetch_server_list,
};
use consts::OFFICIAL_AUTH_SERVER;
use iced::{
    Alignment, Command, Length,
    alignment::{Horizontal, Vertical},
    widget::{
        Image, button, column, container, horizontal_rule, image, image::Handle, row,
        scrollable, text, text_input, tooltip, tooltip::Position,
    },
};
use rust_i18n::t;
use std::{borrow::Cow, cmp::min, collections::HashMap, time::Duration};
use tracing::debug;
use url::Url;
use veloren_query_server::{client::QueryClient, proto::ServerInfo as QueryServerInfo};
use veloren_serverbrowser_api::{FieldContent, GameServer};

pub const SERVER_BROWSER_PING_REFRESH: Duration = Duration::from_secs(20);

#[derive(Clone, Debug)]
pub struct ServerBrowserEntry {
    server: GameServer,
    ping: Option<Duration>,
    server_info: Option<QueryServerInfo>,
    query_client: SkipDebugClone<Option<QueryClient>>,
    source: ServerEntrySource,
}

/// Where a `ServerBrowserEntry` came from - the curated list fetched from
/// `OFFICIAL_SERVER_LIST`, or typed in by the player. Only `Custom` entries are
/// removable and get the "unverified" badge - see Matías's request to let players
/// add a server by IP/DNS name, plus his concern about accidentally connecting to a
/// vanilla Veloren server or an unrelated fork. Today this can only confirm the
/// address speaks the query-server protocol at all (see `AddCustomServerSubmit`);
/// confirming it's specifically a Xindeler-compatible build needs a protocol-level
/// identity field that doesn't exist yet (tracked separately with the
/// xindeler-new-horizon side).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ServerEntrySource {
    #[default]
    Official,
    Custom,
}

/// Newtype that skips debug and when the inner type is an option clones will always
/// result in `None`. Needed because `QueryClient` is neither of both.
pub struct SkipDebugClone<T>(pub T);
impl<T> core::fmt::Debug for SkipDebugClone<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SkipDebug").finish()
    }
}
impl<T> Clone for SkipDebugClone<Option<T>> {
    fn clone(&self) -> Self {
        Self(None)
    }
}

impl From<GameServer> for ServerBrowserEntry {
    fn from(server: GameServer) -> Self {
        Self {
            server,
            ping: None,
            server_info: None,
            query_client: SkipDebugClone(None),
            source: ServerEntrySource::Official,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ServerBrowserPanelMessage {
    SelectServerEntry(Option<usize>),
    UpdateServerList(Result<Option<ServerBrowserPanelComponent>>),
    RefreshPing,
    UpdateServerPing {
        server_address: String,
        server_info: Option<QueryServerInfo>,
        ping: Option<Duration>,
        query_client: SkipDebugClone<Option<QueryClient>>,
    },
    SortServers(ServerSortOrder),
    ShowAddServerForm,
    ShowEditServerForm {
        address: String,
        port: u16,
    },
    AddCustomServerNameChanged(String),
    AddCustomServerAddressChanged(String),
    AddCustomServerPortChanged(String),
    AddCustomServerCancelled,
    AddCustomServerSubmit,
    /// Result of pinging the typed address via the query-server protocol - `Some`
    /// means it answered (so it's at least a Veloren-family game server), `None`
    /// means it didn't respond at all.
    AddCustomServerValidated(Option<QueryServerInfo>),
    RemoveCustomServer {
        address: String,
        port: u16,
    },
}

#[derive(Debug, Default, Clone)]
pub struct ServerBrowserPanelComponent {
    servers: Vec<ServerBrowserEntry>,
    selected_index: Option<usize>,
    server_list_fetch_error: bool,
    last_sort_ordering: Option<ServerSortOrder>,
    add_server_form: Option<AddServerForm>,
}

#[derive(Debug, Clone, Default)]
struct AddServerForm {
    name: String,
    address: String,
    port: String,
    state: AddServerFormState,
    /// The address/port this entry was found under before editing, so submitting
    /// can replace it instead of adding a duplicate. `None` means this form is
    /// adding a brand new server rather than editing an existing one.
    editing_original: Option<(String, u16)>,
}

#[derive(Debug, Clone, Default, PartialEq)]
enum AddServerFormState {
    #[default]
    Editing,
    Validating,
    Error(String),
}

impl ServerBrowserPanelComponent {
    /// `custom_servers` are the player's manually-added servers
    /// (`Profile::custom_servers`), merged in regardless of whether the official list
    /// fetch below succeeds, so a player's own servers still show up even when
    /// `OFFICIAL_SERVER_LIST` is unreachable (a real scenario found while building
    /// this: `serverlist.xindeler.com` didn't resolve at all while this was written).
    pub(crate) async fn fetch(custom_servers: Vec<GameServer>) -> Result<Option<Self>> {
        let mut servers: Vec<ServerBrowserEntry>;
        let mut server_list_fetch_error = false;

        if let Ok(server_list) =
            fetch_server_list(format!("{}/v1/servers", OFFICIAL_SERVER_LIST).to_owned())
                .await
        {
            servers = server_list
                .servers
                .into_iter()
                .filter(|x| matches!(x.auth_server.as_str(), OFFICIAL_AUTH_SERVER))
                .map(ServerBrowserEntry::from)
                .collect();
        } else {
            servers = vec![];
            server_list_fetch_error = true;
        }

        servers.extend(custom_servers.into_iter().map(|server| ServerBrowserEntry {
            source: ServerEntrySource::Custom,
            ..ServerBrowserEntry::from(server)
        }));

        Ok(Some(Self {
            servers,
            selected_index: None,
            last_sort_ordering: None,
            server_list_fetch_error,
            add_server_form: None,
        }))
    }

    pub fn view(&self) -> Element<'_, DefaultViewMessage> {
        let top_row = row![].height(Length::Fixed(50.0)).push(
            column![].push(container(
                row![]
                    .push(
                        container(
                            button(Image::new(Handle::from_memory(GLOBE_ICON.to_vec())))
                                .on_press(DefaultViewMessage::ServerBrowserPanel(
                                    ServerBrowserPanelMessage::RefreshPing,
                                )),
                        )
                        .center_x()
                        .center_y()
                        .height(Length::Fill)
                        .width(Length::Shrink)
                        .align_y(Vertical::Center)
                        .padding([0, 0, 0, 12]),
                    )
                    .push(
                        container(
                            text(t!("server_browser_panel.heading"))
                                .style(TextStyle::Primary)
                                .size(16)
                                .font(POPPINS_MEDIUM_FONT),
                        )
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .align_y(Vertical::Center)
                        .padding([1, 0, 0, 8]),
                    )
                    .push(
                        container(
                            button(
                                row![]
                                    .push(
                                        text(
                                            t!("server_browser_panel.get_listed_button"),
                                        )
                                        .size(10),
                                    )
                                    .push(image(Handle::from_memory(
                                        UP_RIGHT_ARROW_ICON.to_vec(),
                                    )))
                                    .spacing(5)
                                    .align_items(Alignment::Center),
                            )
                            .on_press(DefaultViewMessage::Interaction(
                                Interaction::OpenURL(
                                    SERVER_LISTING_REQUEST_URL.to_string(),
                                ),
                            ))
                            .padding([4, 10, 0, 10])
                            .height(Length::Fixed(20.0))
                            .style(ButtonStyle::Chip(BrowserButtonStyle::Extra)),
                        )
                        .height(Length::Fill)
                        .align_y(Vertical::Center)
                        .padding([1, 10, 0, 8]),
                    )
                    .push(
                        container(
                            button(
                                text(t!("server_browser_panel.add_server_button"))
                                    .size(10),
                            )
                            .on_press(DefaultViewMessage::ServerBrowserPanel(
                                ServerBrowserPanelMessage::ShowAddServerForm,
                            ))
                            .padding([4, 10, 0, 10])
                            .height(Length::Fixed(20.0))
                            .style(ButtonStyle::Chip(BrowserButtonStyle::Extra)),
                        )
                        .height(Length::Fill)
                        .align_y(Vertical::Center)
                        .padding([1, 10, 0, 8]),
                    ),
            )),
        );

        let heading_button = |button_text: &str, sort_order: Option<ServerSortOrder>| {
            let mut button = button(
                text(button_text.to_uppercase())
                    .font(POPPINS_MEDIUM_FONT)
                    .size(10)
                    .vertical_alignment(Vertical::Center),
            )
            .padding(0)
            .style(ButtonStyle::ColumnHeading);
            if let Some(order) = sort_order {
                button = button.on_press(DefaultViewMessage::ServerBrowserPanel(
                    ServerBrowserPanelMessage::SortServers(order),
                ))
            }
            button
        };

        const ICON_COLUMN_WIDTH: f32 = 35.0;
        let column_headings = container(
            row![]
                .width(Length::Fill)
                // Spacer heading for icons column
                .push(heading_button("", None).width(Length::Fixed(ICON_COLUMN_WIDTH)))
                .push(
                    heading_button(
                        &t!("server_browser_panel.column_server"),
                        Some(ServerSortOrder::ServerName),
                    )
                    .width(Length::FillPortion(3)),
                )
                .push(
                    heading_button(
                        &t!("server_browser_panel.column_location"),
                        Some(ServerSortOrder::Location),
                    )
                    .width(Length::FillPortion(2)),
                )
                .push(
                    heading_button(
                        &t!("server_browser_panel.column_players"),
                        Some(ServerSortOrder::PlayerCount),
                    )
                    .width(Length::FillPortion(1)),
                )
                .push(
                    heading_button(
                        &t!("server_browser_panel.column_ping"),
                        Some(ServerSortOrder::Ping),
                    )
                    .width(Length::FillPortion(1)),
                ),
        )
        .style(ContainerStyle::ColumnHeading)
        .padding([10, 8])
        .width(Length::Fill);

        let mut server_list = column![];

        let column_cell = |content: &str| {
            text(content)
                .width(Length::FillPortion(3))
                .font(UNIVERSAL_FONT)
                .height(Length::Fill)
                .size(14)
                .vertical_alignment(Vertical::Center)
        };

        for (i, server_entry) in self.servers.iter().enumerate() {
            let ping_icon = match server_entry.ping.map(|p| p.as_millis()) {
                Some(0..=50) => image(Handle::from_memory(PING1_ICON.to_vec())),
                Some(51..=150) => image(Handle::from_memory(PING2_ICON.to_vec())),
                Some(151..=300) => image(Handle::from_memory(PING3_ICON.to_vec())),
                Some(_) => image(Handle::from_memory(PING4_ICON.to_vec())),
                _ => {
                    if server_entry.server.query_port.is_none() {
                        image(Handle::from_memory(PING_NONE_ICON.to_vec()))
                    } else {
                        image(Handle::from_memory(PING_ERROR_ICON.to_vec()))
                    }
                },
            };

            let mut status_icons = row![]
                .spacing(5)
                .height(Length::Fill)
                .align_items(Alignment::Center);

            if !matches!(
                server_entry.server.auth_server.as_str(),
                OFFICIAL_AUTH_SERVER
            ) {
                status_icons = status_icons.push(
                    tooltip(
                        image(Handle::from_memory(KEY_ICON.to_vec()))
                            .height(Length::Fixed(16.0))
                            .width(Length::Fixed(16.0)),
                        text(t!("server_browser_panel.custom_auth_tooltip")).size(14),
                        Position::Right,
                    )
                    .style(ContainerStyle::Tooltip)
                    .gap(5),
                );
            }

            if server_entry.server.official {
                status_icons = status_icons.push(
                    tooltip(
                        image(Handle::from_memory(STAR_ICON.to_vec()))
                            .height(Length::Fixed(16.0))
                            .width(Length::Fixed(16.0)),
                        text(t!("server_browser_panel.official_server_tooltip")).size(14),
                        Position::Right,
                    )
                    .style(ContainerStyle::Tooltip)
                    .gap(5),
                );
            }

            if server_entry.source == ServerEntrySource::Custom {
                // A text badge here doesn't fit `ICON_COLUMN_WIDTH` and bleeds into
                // the name column, so this reuses the same small status-dot pattern
                // `modal_shell`'s eyebrow uses instead of trying to fit a word.
                status_icons = status_icons.push(
                    tooltip(
                        container(text(""))
                            .width(Length::Fixed(8.0))
                            .height(Length::Fixed(8.0))
                            .style(ContainerStyle::StatusDot(ARCANE_500)),
                        text(t!("server_browser_panel.custom_server_tooltip")).size(14),
                        Position::Right,
                    )
                    .style(ContainerStyle::Tooltip)
                    .gap(5),
                );
            }

            let row = row![]
                .width(Length::Fill)
                .align_items(Alignment::Center)
                .push(
                    container(status_icons)
                        .padding([0, 8])
                        .width(Length::Fixed(ICON_COLUMN_WIDTH))
                        .align_x(Horizontal::Right),
                )
                .push(
                    column_cell(
                        // Iced currently doesn't support truncating text widgets to
                        // prevent multi-line overflow so for now we truncate the server
                        // name to a length which doesn't wrap when the XindelerUpdater
                        // window is at its default size
                        &server_entry.server.name
                            [..min(server_entry.server.name.len(), 40)],
                    )
                    .height(Length::Fill)
                    .width(Length::FillPortion(3)),
                )
                .push(
                    column_cell(&server_entry.server.location.as_ref().map_or_else(
                        || t!("server_browser_panel.location_unknown").into_owned(),
                        |country| country.short_name.clone(),
                    ))
                    .width(Length::FillPortion(2)),
                )
                .push(
                    column_cell(&server_entry.server_info.map_or(
                        Cow::Borrowed("?"),
                        |info| {
                            Cow::Owned(format!(
                                "{}/{}",
                                info.players_count, info.player_cap
                            ))
                        },
                    ))
                    .width(Length::FillPortion(1)),
                )
                .push(
                    row![]
                        .spacing(5)
                        .push(
                            container(ping_icon)
                                .height(Length::Fill)
                                .align_y(Vertical::Center),
                        )
                        .push(column_cell(&server_entry.ping.map_or_else(
                            || {
                                if server_entry.server.query_port.is_none() {
                                    "?".to_owned()
                                } else {
                                    t!("server_browser_panel.ping_error").into_owned()
                                }
                            },
                            |x| format!("{}", x.as_millis()),
                        )))
                        .width(Length::FillPortion(1)),
                )
                .padding(0);

            let row_style = if self
                .selected_index
                .is_some_and(|selected_index| selected_index == i)
            {
                ButtonStyle::ListRow(ServerListEntryButtonState::Selected)
            } else {
                ButtonStyle::ListRow(ServerListEntryButtonState::NotSelected)
            };
            let select_row_button = button(container(row).padding([0, 8]))
                .on_press(DefaultViewMessage::ServerBrowserPanel(
                    if self.selected_index == Some(i) {
                        ServerBrowserPanelMessage::SelectServerEntry(None)
                    } else {
                        ServerBrowserPanelMessage::SelectServerEntry(Some(i))
                    },
                ))
                .style(row_style)
                .height(Length::Fixed(30.0))
                .width(Length::Fill)
                .padding(0);

            let mut list_row = row![]
                .align_items(Alignment::Center)
                .push(select_row_button);
            if server_entry.source == ServerEntrySource::Custom {
                let edit_button = tooltip(
                    button(
                        Image::new(Handle::from_memory(PENCIL_ICON.to_vec()))
                            .height(Length::Fixed(13.0))
                            .width(Length::Fixed(13.0)),
                    )
                    .style(ButtonStyle::Transparent)
                    .padding(0)
                    .on_press(
                        DefaultViewMessage::ServerBrowserPanel(
                            ServerBrowserPanelMessage::ShowEditServerForm {
                                address: server_entry.server.address.clone(),
                                port: server_entry.server.port,
                            },
                        ),
                    ),
                    text(t!("server_browser_panel.edit_server_tooltip")).size(14),
                    Position::Left,
                )
                .style(ContainerStyle::Tooltip)
                .gap(5);

                let remove_button = tooltip(
                    button(
                        Image::new(Handle::from_memory(TRASH_ICON.to_vec()))
                            .height(Length::Fixed(13.0))
                            .width(Length::Fixed(13.0)),
                    )
                    .style(ButtonStyle::Transparent)
                    .padding(0)
                    .on_press(
                        DefaultViewMessage::ServerBrowserPanel(
                            ServerBrowserPanelMessage::RemoveCustomServer {
                                address: server_entry.server.address.clone(),
                                port: server_entry.server.port,
                            },
                        ),
                    ),
                    text(t!("server_browser_panel.remove_server_tooltip")).size(14),
                    Position::Left,
                )
                .style(ContainerStyle::Tooltip)
                .gap(5);

                list_row = list_row.push(
                    container(
                        row![]
                            .spacing(6)
                            .align_items(Alignment::Center)
                            .push(edit_button)
                            .push(remove_button),
                    )
                    .width(Length::Fixed(ICON_COLUMN_WIDTH + 14.0))
                    .align_x(Horizontal::Center),
                );
            }

            server_list = server_list.push(list_row);
        }

        let mut col = column![].push(
            container(top_row)
                .width(Length::Fill)
                .height(Length::Shrink)
                .style(ContainerStyle::ChangelogHeader),
        );

        // Only show the hard error state when there's truly nothing to show - the
        // official list can be unreachable while the player still has custom servers
        // of their own (this genuinely happens: `OFFICIAL_SERVER_LIST` was found
        // unreachable while this was built).
        if !self.server_list_fetch_error || !self.servers.is_empty() {
            col = col
                .push(column_headings.height(Length::Shrink))
                .push(scrollable(server_list).height(Length::Fill));

            // If there's a selected server (which there should always be, unless the
            // server list API returned no servers) show details for that
            // server.
            let selected_server = self.selected_index.and_then(|x| self.servers.get(x));

            let discord_origin = url::Origin::Tuple(
                "https".to_string(),
                url::Host::Domain(String::from("discord.gg")),
                443,
            );
            let reddit_origin = url::Origin::Tuple(
                "https".to_string(),
                url::Host::Domain(String::from("reddit.com")),
                443,
            );
            let youtube_origin = url::Origin::Tuple(
                "https".to_string(),
                url::Host::Domain(String::from("youtube.com")),
                443,
            );

            if let Some(server) = selected_server {
                col = col
                    .push(
                        container(horizontal_rule(8))
                            .width(Length::Fill)
                            .padding([5, 20]),
                    )
                    .push(
                        container(scrollable(container({
                            let mut fields = server
                                .server
                                .extra
                                .clone()
                                .into_iter()
                                .collect::<Vec<_>>();
                            fields.sort_by(|a, b| a.0.cmp(&b.0));
                            let mut extras = row![].spacing(10);
                            for (id, field) in fields {
                                // TODO: Recognise common IDs, give them a custom icon
                                match field.content {
                                    FieldContent::Text(c) => {
                                        let container = match id.as_str() {
                                            "email" => container(
                                                text(t!(
                                                    "server_browser_panel.email_field",
                                                    value = c
                                                ))
                                                .size(12),
                                            )
                                            .padding([2, 10, 2, 10])
                                            .style(ContainerStyle::ExtraBrowser),
                                            _ => container(
                                                text(t!(
                                                    "server_browser_panel.extra_field",
                                                    name = field.name,
                                                    value = c
                                                ))
                                                .size(14),
                                            )
                                            .padding([2, 10, 2, 10])
                                            .style(ContainerStyle::ExtraBrowser),
                                        };
                                        extras = extras.push(container);
                                    },
                                    FieldContent::Url(c) => {
                                        let mut button = button(
                                            row![]
                                                .push(text(field.name).size(12))
                                                .push(image(Handle::from_memory(
                                                    UP_RIGHT_ARROW_ICON.to_vec(),
                                                )))
                                                .spacing(5)
                                                .align_items(Alignment::Center),
                                        )
                                        .on_press(DefaultViewMessage::Interaction(
                                            Interaction::OpenURL(c.clone()),
                                        ))
                                        .padding([2, 10, 2, 10])
                                        .height(Length::Fixed(20.0));
                                        let button_style = match id.as_str() {
                                            "discord"
                                                if Url::parse(&c)
                                                    .map(|u| u.origin() == discord_origin)
                                                    .unwrap_or(false) =>
                                            {
                                                ButtonStyle::Chip(
                                                    BrowserButtonStyle::Discord,
                                                )
                                            },
                                            "reddit"
                                                if Url::parse(&c)
                                                    .map(|u| u.origin() == reddit_origin)
                                                    .unwrap_or(false) =>
                                            {
                                                ButtonStyle::Chip(
                                                    BrowserButtonStyle::Reddit,
                                                )
                                            },
                                            "youtube"
                                                if Url::parse(&c)
                                                    .map(|u| u.origin() == youtube_origin)
                                                    .unwrap_or(false) =>
                                            {
                                                ButtonStyle::Chip(
                                                    BrowserButtonStyle::Youtube,
                                                )
                                            },
                                            "mastodon" => ButtonStyle::Chip(
                                                BrowserButtonStyle::Mastodon,
                                            ),
                                            "twitch" => ButtonStyle::Chip(
                                                BrowserButtonStyle::Twitch,
                                            ),
                                            _ => ButtonStyle::Chip(
                                                BrowserButtonStyle::Extra,
                                            ),
                                        };
                                        button = button.style(button_style);
                                        extras = extras.push(button);
                                    },
                                    _ => {},
                                };
                            }
                            let queried_info =
                                if let Some(query_info) = &server.server_info {
                                    let battlemode = match query_info.battlemode {
                                        veloren_query_server::proto::ServerBattleMode::GlobalPvP => {
                                            t!("server_browser_panel.battlemode_global_pvp")
                                        },
                                        veloren_query_server::proto::ServerBattleMode::GlobalPvE => {
                                            t!("server_browser_panel.battlemode_global_pve")
                                        },
                                        veloren_query_server::proto::ServerBattleMode::PerPlayer => {
                                            t!("server_browser_panel.battlemode_per_player")
                                        },
                                    };

                                    column![
                                        text(t!(
                                            "server_browser_panel.battlemode",
                                            mode = battlemode
                                        ))
                                        .size(14),
                                        text(t!(
                                            "server_browser_panel.game_version",
                                            hash = format!("{:x}", query_info.git_hash)
                                        ))
                                        .size(14),
                                    ]
                                    .spacing(5)
                                } else {
                                    column![
                                        text(t!("server_browser_panel.no_query_protocol"))
                                            .size(14)
                                    ]
                                };

                            column![]
                                .spacing(5)
                                .width(Length::Fill)
                                .push(
                                    row![]
                                        .spacing(10)
                                        .push(
                                            text(&server.server.name)
                                                .font(UNIVERSAL_FONT)
                                                .size(14),
                                        )
                                        .push(
                                            text(display_gameserver_address(
                                                &server.server,
                                            ))
                                            .size(14)
                                            .font(UNIVERSAL_FONT)
                                            .style(TextStyle::Accent),
                                        ),
                                )
                                .push(
                                    text(t!("server_browser_panel.description_label"))
                                        .font(UNIVERSAL_FONT)
                                        .size(14),
                                )
                                .push(
                                    text(&server.server.description)
                                        .font(UNIVERSAL_FONT)
                                        .size(14),
                                )
                                .push(queried_info)
                                .push(extras)
                        }).width(Length::Fill)))
                        .height(Length::Fixed(160.0))
                        .padding([0, 0, 0, 40]),
                    );
            }
        } else {
            col = col.push(
                container(
                    text(t!("server_browser_panel.fetch_error"))
                        .size(14)
                        .style(TextStyle::Danger),
                )
                .padding(20)
                .align_x(Horizontal::Center),
            )
        }

        let server_browser_container = container(col)
            .height(Length::Fill)
            .width(Length::Fill)
            .style(ContainerStyle::Dark);
        server_browser_container.into()
    }

    /// The "add a custom server" dialog, when it's open - `default.rs` layers this on
    /// top of the whole window, same as the update prompts.
    pub fn add_server_modal(&self) -> Option<Element<'_, DefaultViewMessage>> {
        let form = self.add_server_form.as_ref()?;

        let is_validating = form.state == AddServerFormState::Validating;
        let is_editing = form.editing_original.is_some();

        let mut body = column![]
            .spacing(10)
            .push(
                text_input(
                    &t!("server_browser_panel.add_server_name_placeholder"),
                    &form.name,
                )
                .on_input(|name| {
                    DefaultViewMessage::ServerBrowserPanel(
                        ServerBrowserPanelMessage::AddCustomServerNameChanged(name),
                    )
                })
                .padding(7)
                .size(13),
            )
            .push(
                text_input(
                    &t!("server_browser_panel.add_server_address_placeholder"),
                    &form.address,
                )
                .on_input(|address| {
                    DefaultViewMessage::ServerBrowserPanel(
                        ServerBrowserPanelMessage::AddCustomServerAddressChanged(address),
                    )
                })
                .padding(7)
                .size(13),
            )
            .push(
                text_input(
                    &t!("server_browser_panel.add_server_port_placeholder"),
                    &form.port,
                )
                .on_input(|port| {
                    DefaultViewMessage::ServerBrowserPanel(
                        ServerBrowserPanelMessage::AddCustomServerPortChanged(port),
                    )
                })
                .padding(7)
                .size(13),
            );

        match &form.state {
            AddServerFormState::Error(message) => {
                body = body.push(text(message.clone()).size(12).style(TextStyle::Danger));
            },
            AddServerFormState::Validating => {
                body = body.push(
                    text(t!("server_browser_panel.add_server_validating"))
                        .size(12)
                        .style(TextStyle::Muted),
                );
            },
            AddServerFormState::Editing => {},
        }

        let confirm_label = if is_editing {
            t!("server_browser_panel.add_server_save")
        } else {
            t!("server_browser_panel.add_server_confirm")
        };
        let mut add_button =
            button(text(confirm_label).font(POPPINS_MEDIUM_FONT).size(14))
                .style(ButtonStyle::Primary)
                .padding([10, 22]);
        if !is_validating {
            add_button = add_button.on_press(DefaultViewMessage::ServerBrowserPanel(
                ServerBrowserPanelMessage::AddCustomServerSubmit,
            ));
        }

        let cancel_button = button(
            text(t!("server_browser_panel.add_server_cancel"))
                .font(POPPINS_MEDIUM_FONT)
                .size(14),
        )
        .style(ButtonStyle::Ghost)
        .padding([10, 18])
        .on_press(DefaultViewMessage::ServerBrowserPanel(
            ServerBrowserPanelMessage::AddCustomServerCancelled,
        ));

        let actions = row![].spacing(10).push(cancel_button).push(add_button);

        let title = if is_editing {
            t!("server_browser_panel.add_server_edit_title")
        } else {
            t!("server_browser_panel.add_server_title")
        };

        Some(modal_shell(
            ARCANE_500,
            t!("server_browser_panel.add_server_eyebrow"),
            title,
            body.into(),
            Some(actions.into()),
        ))
    }

    pub fn update(
        &mut self,
        msg: ServerBrowserPanelMessage,
        active_profile: &Profile,
    ) -> Option<Command<DefaultViewMessage>> {
        match msg {
            ServerBrowserPanelMessage::UpdateServerList(result) => match result {
                Ok(Some(server_browser)) => {
                    *self = server_browser;
                    if !self.servers.is_empty() {
                        // Why is there no simple `Command::message` ??
                        Some(Command::perform(async {}, |()| {
                            DefaultViewMessage::ServerBrowserPanel(
                                ServerBrowserPanelMessage::RefreshPing,
                            )
                        }))
                    } else {
                        None
                    }
                },
                Ok(None) => None,
                Err(e) => {
                    tracing::trace!("Failed to update server list: {}", e);
                    None
                },
            },
            ServerBrowserPanelMessage::UpdateServerPing {
                server_address,
                server_info,
                ping,
                query_client,
            } => {
                debug!(?ping, ?server_address, "Received ping result for server");

                if let Some(server) = self
                    .servers
                    .iter_mut()
                    .find(|x| x.server.address == server_address)
                {
                    server.ping = ping;
                    server.server_info = server_info;
                    server.query_client = query_client;
                };

                self.sort_servers(self.last_sort_ordering.unwrap_or_default());

                None
            },
            ServerBrowserPanelMessage::RefreshPing => Some(Command::batch(
                self.servers.iter_mut().filter_map(|server| {
                    let query_client = server.query_client.0.take();
                    let query_port = server.server.query_port?;
                    let server_address = server.server.address.clone();
                    let server_address2 = server.server.address.clone();

                    Some(Command::perform(
                        async move {
                            let mut query_client = match query_client {
                                Some(client) => client,
                                None => {
                                    crate::net::ping::create_client(
                                        &server_address2,
                                        query_port,
                                    )
                                    .await?
                                },
                            };
                            debug!(?server_address2, "Querying server");

                            let res =
                                crate::net::ping::perform_ping(&mut query_client).await;
                            Some((res, query_client))
                        },
                        move |res| {
                            let (query_client, server_info, ping) =
                                if let Some((res, query_client)) = res {
                                    let (ping, server_info) = res
                                        .inspect_err(|error| {
                                            debug!(
                                                ?server_address,
                                                ?error,
                                                "Failed to query server"
                                            )
                                        })
                                        .map_or((None, None), |(info, ping)| {
                                            (Some(info), Some(ping))
                                        });
                                    (Some(query_client), server_info, ping)
                                } else {
                                    (None, None, None)
                                };

                            DefaultViewMessage::ServerBrowserPanel(
                                ServerBrowserPanelMessage::UpdateServerPing {
                                    server_address,
                                    server_info,
                                    ping,
                                    query_client: SkipDebugClone(query_client),
                                },
                            )
                        },
                    ))
                }),
            )),
            ServerBrowserPanelMessage::SelectServerEntry(index) => {
                self.selected_index = index;
                let selected_server = index.and_then(|index| {
                    self.servers
                        .get(index)
                        .map(|x| display_gameserver_address(&x.server))
                });

                Some(Command::perform(async {}, move |()| {
                    DefaultViewMessage::GamePanel(
                        GamePanelMessage::ServerBrowserServerChanged(selected_server),
                    )
                }))
            },
            ServerBrowserPanelMessage::SortServers(order) => {
                self.sort_servers(order);
                self.last_sort_ordering = Some(order);
                None
            },
            ServerBrowserPanelMessage::ShowAddServerForm => {
                self.add_server_form = Some(AddServerForm::default());
                None
            },
            ServerBrowserPanelMessage::ShowEditServerForm { address, port } => {
                if let Some(entry) = self.servers.iter().find(|entry| {
                    entry.source == ServerEntrySource::Custom
                        && entry.server.address == address
                        && entry.server.port == port
                }) {
                    self.add_server_form = Some(AddServerForm {
                        name: entry.server.name.clone(),
                        address: entry.server.address.clone(),
                        port: entry.server.port.to_string(),
                        state: AddServerFormState::Editing,
                        editing_original: Some((address, port)),
                    });
                }
                None
            },
            ServerBrowserPanelMessage::AddCustomServerCancelled => {
                self.add_server_form = None;
                None
            },
            ServerBrowserPanelMessage::AddCustomServerNameChanged(name) => {
                if let Some(form) = &mut self.add_server_form {
                    form.name = name;
                }
                None
            },
            ServerBrowserPanelMessage::AddCustomServerAddressChanged(address) => {
                if let Some(form) = &mut self.add_server_form {
                    form.address = address;
                    form.state = AddServerFormState::Editing;
                }
                None
            },
            ServerBrowserPanelMessage::AddCustomServerPortChanged(port) => {
                if let Some(form) = &mut self.add_server_form {
                    form.port = port;
                    form.state = AddServerFormState::Editing;
                }
                None
            },
            ServerBrowserPanelMessage::AddCustomServerSubmit => {
                let Some(form) = &mut self.add_server_form else {
                    return None;
                };

                let address = form.address.trim().to_owned();
                if address.is_empty() {
                    form.state = AddServerFormState::Error(
                        t!("server_browser_panel.add_server_error_empty").into_owned(),
                    );
                    return None;
                }
                if !form.port.trim().is_empty()
                    && form.port.trim().parse::<u16>().is_err()
                {
                    form.state = AddServerFormState::Error(
                        t!("server_browser_panel.add_server_error_port").into_owned(),
                    );
                    return None;
                }

                form.state = AddServerFormState::Validating;

                Some(Command::perform(
                    async move {
                        let mut client = crate::net::ping::create_client(
                            &address,
                            net::DEFAULT_QUERY_PORT,
                        )
                        .await?;
                        crate::net::ping::perform_ping(&mut client)
                            .await
                            .ok()
                            .map(|(_ping, info)| info)
                    },
                    |info| {
                        DefaultViewMessage::ServerBrowserPanel(
                            ServerBrowserPanelMessage::AddCustomServerValidated(info),
                        )
                    },
                ))
            },
            ServerBrowserPanelMessage::AddCustomServerValidated(info) => {
                let Some(form) = &self.add_server_form else {
                    return None;
                };

                match info {
                    Some(info) => {
                        debug!(?info, "Validated custom server, saving it");

                        let address = form.address.trim().to_owned();
                        let port = form
                            .port
                            .trim()
                            .parse::<u16>()
                            .unwrap_or(net::DEFAULT_GAME_PORT);
                        let name = form.name.trim();
                        let name = if name.is_empty() {
                            address.clone()
                        } else {
                            name.to_owned()
                        };
                        let editing_original = form.editing_original.clone();

                        let game_server = GameServer {
                            name,
                            address,
                            port,
                            description: String::new(),
                            location: None,
                            auth_server: OFFICIAL_AUTH_SERVER.to_owned(),
                            query_port: Some(net::DEFAULT_QUERY_PORT),
                            channel: None,
                            official: false,
                            extra: HashMap::new(),
                        };

                        let mut profile = active_profile.clone();
                        // Editing an existing entry replaces it (by its original
                        // address/port) instead of leaving a stale duplicate behind.
                        if let Some((old_address, old_port)) = &editing_original {
                            profile.custom_servers.retain(|s| {
                                !(&s.address == old_address && &s.port == old_port)
                            });
                            self.servers.retain(|entry| {
                                !(entry.source == ServerEntrySource::Custom
                                    && &entry.server.address == old_address
                                    && &entry.server.port == old_port)
                            });
                        }

                        let already_added = profile.custom_servers.iter().any(|s| {
                            s.address == game_server.address && s.port == game_server.port
                        });
                        if !already_added {
                            profile.custom_servers.push(game_server.clone());
                        }

                        self.servers.push(ServerBrowserEntry {
                            source: ServerEntrySource::Custom,
                            ..ServerBrowserEntry::from(game_server)
                        });
                        self.sort_servers(self.last_sort_ordering.unwrap_or_default());
                        self.add_server_form = None;
                        self.selected_index = None;

                        Some(Command::perform(
                            async { Action::UpdateProfile(profile) },
                            DefaultViewMessage::Action,
                        ))
                    },
                    None => {
                        if let Some(form) = &mut self.add_server_form {
                            form.state = AddServerFormState::Error(
                                t!("server_browser_panel.add_server_error_unreachable")
                                    .into_owned(),
                            );
                        }
                        None
                    },
                }
            },
            ServerBrowserPanelMessage::RemoveCustomServer { address, port } => {
                let mut profile = active_profile.clone();
                profile
                    .custom_servers
                    .retain(|s| !(s.address == address && s.port == port));

                self.servers.retain(|entry| {
                    !(entry.source == ServerEntrySource::Custom
                        && entry.server.address == address
                        && entry.server.port == port)
                });
                self.selected_index = None;

                Some(Command::perform(
                    async { Action::UpdateProfile(profile) },
                    DefaultViewMessage::Action,
                ))
            },
        }
    }

    fn sort_servers(&mut self, order: ServerSortOrder) {
        match order {
            ServerSortOrder::Default => self.servers.sort_unstable_by_key(|x| {
                (
                    !x.server.official,
                    x.ping.or(Some(Duration::MAX)),
                    x.server.name.clone(),
                )
            }),
            ServerSortOrder::PlayerCount => {
                self.servers.sort_unstable_by(|entry_a, entry_b| {
                    let cnt = |e: &ServerBrowserEntry| {
                        e.server_info.map(|info| info.players_count)
                    };

                    cnt(entry_b).cmp(&cnt(entry_a))
                })
            },
            ServerSortOrder::Ping => self
                .servers
                .sort_unstable_by_key(|x| x.ping.or(Some(Duration::MAX))),
            ServerSortOrder::ServerName => {
                self.servers.sort_unstable_by_key(|x| x.server.name.clone())
            },
            ServerSortOrder::Location => self.servers.sort_unstable_by_key(|x| {
                x.server
                    .location
                    .as_ref()
                    .map_or("".to_owned(), |country| country.short_name.clone())
            }),
        }
    }
}

fn display_gameserver_address(gameserver: &GameServer) -> String {
    if gameserver.port == net::DEFAULT_GAME_PORT {
        gameserver.address.clone()
    } else {
        format!("{}:{}", gameserver.address, gameserver.port)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub enum ServerSortOrder {
    #[default]
    Default,
    PlayerCount,
    ServerName,
    Location,
    Ping,
}
