use crate::{
    assets::{BOOK_ICON, CHAT_ICON, UP_RIGHT_ARROW_ICON, USER_ICON, XINDELER_LOGO},
    gui::{
        style::button::ButtonStyle,
        views::default::{DefaultViewMessage, Interaction},
        widget::*,
    },
};
use iced::{
    Alignment, Length,
    alignment::Vertical,
    widget::{Image, button, column, container, image::Handle, row, text, text::Shaping},
};

#[derive(Clone, Default, Debug)]
pub struct LogoPanelComponent {}

impl LogoPanelComponent {
    pub fn view(&self) -> Element<'_, DefaultViewMessage> {
        let col = column![]
            .push(Image::new(Handle::from_memory(XINDELER_LOGO.to_vec())))
            .push(
                container(
                    column![]
                        .spacing(2)
                        .push(link_widget(
                            BOOK_ICON,
                            "https://book.xindeler.com/",
                            "Game Manual",
                        ))
                        .push(link_widget(
                            CHAT_ICON,
                            "https://discord.gg/hgdhHY6vw",
                            "Community",
                        ))
                        .push(link_widget(
                            USER_ICON,
                            "https://xindeler.com/account/",
                            "Create Account",
                        )),
                    // Donate link removed for now: Xindeler doesn't have its own
                    // donation page yet.
                )
                .padding([32, 0, 0, 0]),
            );

        let container: Container<'_, DefaultViewMessage> = container(col).padding(20);
        container.into()
    }
}

fn link_widget<'a>(
    image_bytes: &[u8],
    url: &'a str,
    link_text: &'a str,
) -> Element<'a, DefaultViewMessage> {
    container(
        button(
            row![]
                .align_items(Alignment::Center)
                .push(
                    container(
                        Image::new(Handle::from_memory(image_bytes.to_vec()))
                            .height(Length::Fixed(20.0))
                            .width(Length::Fixed(20.0)),
                    )
                    .align_y(Vertical::Center),
                )
                .push(
                    container(
                        text(link_text)
                            .font(crate::assets::POPPINS_MEDIUM_FONT)
                            .size(13)
                            .shaping(Shaping::Advanced),
                    )
                    .align_y(Vertical::Center),
                )
                .push(
                    container(Image::new(Handle::from_memory(
                        UP_RIGHT_ARROW_ICON.to_vec(),
                    )))
                    .align_y(Vertical::Center),
                )
                .spacing(10),
        )
        .padding([8, 10])
        .width(Length::Fill)
        .on_press(DefaultViewMessage::Interaction(Interaction::OpenURL(
            url.to_string(),
        )))
        .style(ButtonStyle::Ghost),
    )
    .height(Length::Shrink)
    .into()
}
