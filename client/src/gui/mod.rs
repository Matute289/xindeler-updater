mod background_image;
pub mod components;
mod custom_widgets;
mod rss_feed;
mod style;
mod subscriptions;
mod views;
mod widget;

use std::borrow::Cow;

#[cfg(feature = "bundled_font")]
use crate::assets::UNIVERSAL_FONT_BYTES;
use crate::{
    Result,
    assets::{
        POPPINS_BOLD_FONT_BYTES, POPPINS_FONT_BYTES, POPPINS_LIGHT_FONT_BYTES,
        POPPINS_MEDIUM_FONT_BYTES,
    },
    cli::CmdLine,
    gui::{style::XindelerUpdaterTheme, widget::*},
    profiles::Profile,
};
use iced::{Application, Command, Settings, Size, Subscription};
use views::{
    Action,
    default::{DefaultView, DefaultViewMessage},
};

/// Starts the GUI and won't return unless an error occurs
pub fn run(cmd: CmdLine) -> Result<()> {
    Ok(XindelerUpdater::run(settings(cmd))?)
}

#[derive(Debug, Clone)]
pub struct XindelerUpdater {
    pub default_view: DefaultView,
    pub active_profile: Profile,
}

impl XindelerUpdater {
    const APP_ID: &'static str = "com.xindeler.xindeler-updater";

    pub fn new(active_profile: Profile) -> Self {
        Self {
            default_view: DefaultView::default(),
            active_profile,
        }
    }
}

#[allow(clippy::enum_variant_names, clippy::large_enum_variant)]
#[derive(Clone, Debug)]
pub enum Message {
    Loaded,
    #[allow(dead_code)]
    Saved(Result<()>),

    // Views
    DefaultViewMessage(DefaultViewMessage),
}

impl Application for XindelerUpdater {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = XindelerUpdaterTheme;
    type Flags = CmdLine;

    fn new(_flags: CmdLine) -> (Self, Command<Message>) {
        #[cfg(windows)]
        crate::windows::hide_non_inherited_console();

        (
            XindelerUpdater::new(Profile::load()),
            Command::perform(async {}, |_| Message::Loaded),
        )
    }

    fn title(&self) -> String {
        format!("XindelerUpdater v{}", env!("CARGO_PKG_VERSION"))
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Loaded => {
                return self
                    .default_view
                    .update(DefaultViewMessage::Query, &self.active_profile)
                    .map(Message::DefaultViewMessage);
            },
            Message::Saved(_) => {},

            // Views
            Message::DefaultViewMessage(msg) => {
                if let DefaultViewMessage::Action(Action::UpdateProfile(profile)) = &msg {
                    self.active_profile = profile.clone();
                    self.active_profile.reload_wgpu_backends();
                    self.active_profile.reload_wgpu_devices();

                    return Command::perform(
                        Profile::save(self.active_profile.clone()),
                        Message::Saved,
                    );
                }

                return self
                    .default_view
                    .update(msg, &self.active_profile)
                    .map(Message::DefaultViewMessage);
            },
        }

        Command::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        self.default_view
            .view(&self.active_profile)
            .map(Message::DefaultViewMessage)
    }

    fn theme(&self) -> Self::Theme {
        XindelerUpdaterTheme {}
    }

    fn subscription(&self) -> Subscription<Message> {
        self.default_view
            .subscription()
            .map(Message::DefaultViewMessage)
    }
}

fn settings(cmd: CmdLine) -> Settings<CmdLine> {
    use iced::window::{Settings as Window, icon};
    let icon = image::load_from_memory(crate::assets::XINDELER_ICON).unwrap();

    #[cfg_attr(not(target_os = "linux"), expect(unused_mut))]
    let mut window_settings = Window {
        size: Size::new(1050.0, 720.0),
        resizable: true,
        decorations: true,
        icon: Some(
            icon::from_rgba(icon.to_rgba8().into_raw(), icon.width(), icon.height())
                .unwrap(),
        ),
        min_size: Some(Size::new(400.0, 250.0)),
        ..Default::default()
    };

    #[cfg(target_os = "linux")]
    {
        window_settings.platform_specific.application_id = XindelerUpdater::APP_ID.to_string();
    }

    Settings {
        window: window_settings,
        flags: cmd,
        default_font: crate::assets::POPPINS_FONT,
        default_text_size: 20.0.into(),
        antialiasing: true,
        id: Some(XindelerUpdater::APP_ID.to_string()),
        fonts: vec![
            #[cfg(feature = "bundled_font")]
            Cow::Borrowed(UNIVERSAL_FONT_BYTES),
            Cow::Borrowed(POPPINS_FONT_BYTES),
            Cow::Borrowed(POPPINS_BOLD_FONT_BYTES),
            Cow::Borrowed(POPPINS_MEDIUM_FONT_BYTES),
            Cow::Borrowed(POPPINS_LIGHT_FONT_BYTES),
        ],
    }
}
