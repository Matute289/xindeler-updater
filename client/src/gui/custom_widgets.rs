use crate::{
    assets::{POPPINS_BOLD_FONT, POPPINS_MEDIUM_FONT},
    gui::{
        style::{container::ContainerStyle, text::TextStyle},
        widget::*,
    },
};
use iced::{
    Alignment, Color, Length,
    alignment::{Horizontal, Vertical},
    widget::{column, container, horizontal_rule, horizontal_space, row, text},
};

pub(crate) fn heading_with_rule<'a, T: 'a>(heading_text: &'a str) -> Element<'a, T> {
    container(
        row![]
            .align_items(Alignment::Center)
            .push(container(horizontal_rule(8)).width(Length::Fixed(13.0)))
            .push(
                container(text(heading_text).font(POPPINS_BOLD_FONT).size(15))
                    .padding([0, 7]),
            )
            .push(container(horizontal_rule(8)).width(Length::Fill)),
    )
    .into()
}

/// The shared modal shell used by both the game-update prompt and the launcher-update
/// dialog - see docs/design/ui-refresh-spec.md §6.1. Backdrop dims the whole window,
/// the card is left-aligned text with right-aligned actions (the single biggest
/// change from the old centred-everything layout).
pub(crate) fn modal_shell<'a, Message: 'a>(
    eyebrow_color: Color,
    eyebrow_label: &'static str,
    title: impl Into<String>,
    body: Element<'a, Message>,
    actions: Option<Element<'a, Message>>,
) -> Element<'a, Message> {
    let title = title.into();
    let eyebrow = row![]
        .spacing(8)
        .align_items(Alignment::Center)
        .push(
            container(text(""))
                .width(Length::Fixed(8.0))
                .height(Length::Fixed(8.0))
                .style(ContainerStyle::StatusDot(eyebrow_color)),
        )
        .push(
            text(eyebrow_label)
                .font(POPPINS_MEDIUM_FONT)
                .size(10)
                .style(TextStyle::Muted),
        );

    let mut card = column![]
        .spacing(0)
        .push(eyebrow)
        .push(container(text("")).height(Length::Fixed(12.0)))
        .push(
            text(title)
                .font(POPPINS_BOLD_FONT)
                .size(20)
                .style(TextStyle::Primary),
        )
        .push(container(text("")).height(Length::Fixed(8.0)))
        .push(body)
        .push(container(text("")).height(Length::Fixed(24.0)));

    if let Some(actions) = actions {
        card =
            card.push(row![horizontal_space(), actions].align_items(Alignment::Center));
    }

    container(
        container(card)
            .style(ContainerStyle::ModalDialog)
            .width(Length::Fixed(420.0))
            .padding([24, 28, 20, 28]),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .align_x(Horizontal::Center)
    .align_y(Vertical::Center)
    .style(ContainerStyle::ModalBackdrop)
    .into()
}
