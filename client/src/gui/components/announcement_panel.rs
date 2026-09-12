use crate::{
    Result,
    assets::{POPPINS_MEDIUM_FONT, UP_RIGHT_ARROW_ICON},
    consts::{SUPPORTED_SERVER_API_VERSION, XINDELER_UPDATER_RELEASE_URL},
    gui::{
        style::{button::ButtonStyle, container::ContainerStyle, text::TextStyle},
        views::default::{DefaultViewMessage, Interaction},
        widget::*,
    },
    net,
};
use iced::{
    Alignment, Command, Length,
    alignment::Vertical,
    widget::{button, column, container, image, image::Handle, row, text},
};
use rust_i18n::t;
use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Clone, Debug)]
pub enum AnnouncementPanelMessage {
    FetchAnnouncement(Result<AnnouncementPanelComponent>),
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct AnnouncementPanelComponent {
    pub announcement_message: Option<String>,
    pub announcement_last_change: chrono::DateTime<chrono::Utc>,
    pub api_version: Option<u32>,
}

impl AnnouncementPanelComponent {
    pub async fn fetch(
        api_version_url: String,
        announcement_url: String,
    ) -> Result<Self> {
        #[derive(Deserialize)]
        pub struct Version {
            version: u32,
        }

        #[derive(Deserialize)]
        pub struct Announcement {
            message: Option<String>,
            last_change: chrono::DateTime<chrono::Utc>,
        }

        debug!("Announcement fetching...");

        let version = net::query(api_version_url).await?.json::<Version>().await?;
        let announcement = net::query(announcement_url)
            .await?
            .json::<Announcement>()
            .await?;

        Ok(AnnouncementPanelComponent {
            announcement_message: announcement.message,
            announcement_last_change: announcement.last_change,
            api_version: Some(version.version),
        })
    }

    pub fn update(
        &mut self,
        msg: AnnouncementPanelMessage,
    ) -> Option<Command<DefaultViewMessage>> {
        match msg {
            AnnouncementPanelMessage::FetchAnnouncement(result) => match result {
                Ok(announcement) => {
                    *self = announcement;
                    None
                },
                Err(e) => {
                    tracing::trace!("Failed to fetch announcement: {}", e);
                    None
                },
            },
        }
    }

    pub fn view(&self) -> Element<'_, DefaultViewMessage> {
        let update = match self.api_version {
            Some(version) => SUPPORTED_SERVER_API_VERSION != version,
            None => false,
        };
        let rowtext = match (update, &self.announcement_message) {
            (false, None) => {
                return row![].into();
            },
            (true, None) => t!("announcement_panel.outdated").into_owned(),
            (false, Some(msg)) => {
                let date: chrono::DateTime<chrono::Local> =
                    self.announcement_last_change.into();
                t!(
                    "announcement_panel.news",
                    date = date.format("%Y-%m-%d %H:%M"),
                    message = msg
                )
                .into_owned()
            },
            (true, Some(msg)) => {
                t!("announcement_panel.outdated_with_news", message = msg).into_owned()
            },
        };

        let mut content_row = row![
            container(
                Text::new(rowtext)
                    .size(13)
                    .style(TextStyle::Accent)
                    .font(POPPINS_MEDIUM_FONT),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .align_y(Vertical::Center)
            .padding([0, 0, 0, 16]),
        ];
        if update {
            content_row = content_row.push(
                container(
                    button(
                        row![
                            text(t!("announcement_panel.download_button")).size(11),
                            image(Handle::from_memory(UP_RIGHT_ARROW_ICON.to_vec(),))
                        ]
                        .spacing(5)
                        .align_items(Alignment::Center),
                    )
                    .on_press(DefaultViewMessage::Interaction(Interaction::OpenURL(
                        XINDELER_UPDATER_RELEASE_URL.to_string(),
                    )))
                    .padding([0, 12])
                    .height(Length::Fixed(26.0))
                    .style(ButtonStyle::Secondary),
                )
                .padding([0, 20, 0, 0])
                .height(Length::Fill)
                .align_y(Vertical::Center)
                .width(Length::Shrink),
            );
        }

        let top_row = row![column![
            container(content_row.height(Length::Fill)).align_y(Vertical::Center),
        ]]
        .height(Length::Fixed(44.0));

        let col = column![].push(
            container(top_row)
                .width(Length::Fill)
                .style(ContainerStyle::Announcement),
        );

        let announcement_container = container(col);
        announcement_container.into()
    }
}
