#![allow(dead_code)]
use crate::gui::XindelerUpdaterTheme;

pub type Element<'a, Message> = iced::Element<'a, Message, XindelerUpdaterTheme>;
pub type Container<'a, Message> = iced::widget::Container<'a, Message, XindelerUpdaterTheme>;
pub type Button<'a, Message> = iced::widget::Button<'a, Message, XindelerUpdaterTheme>;
pub type ProgressBar = iced::widget::ProgressBar<XindelerUpdaterTheme>;
pub type PickList<'a, T, L, V, Message> =
    iced::widget::PickList<'a, T, L, V, Message, XindelerUpdaterTheme>;
pub type TextInput<'a, Message> = iced::widget::TextInput<'a, Message, XindelerUpdaterTheme>;
pub type Rule = iced::widget::Rule<XindelerUpdaterTheme>;
pub type Text<'a> = iced::widget::Text<'a, XindelerUpdaterTheme>;
pub type Tooltip<'a, Message> = iced::widget::Tooltip<'a, Message, XindelerUpdaterTheme>;
