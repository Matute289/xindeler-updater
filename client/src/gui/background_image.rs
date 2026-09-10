//! A widget that paints an image behind another element, filling the same
//! bounds. Needed because this iced version (0.12) has no `Stack` widget and
//! `container::Appearance` only supports solid colors/gradients as a
//! background, not images.
use iced::{
    Element, Length, Size,
    advanced::{
        Clipboard, Layout, Shell, Widget,
        layout::{Limits, Node},
        mouse, overlay, renderer,
        widget::{Operation, Tree, tree},
    },
    event::{self, Event},
};

/// Wraps `foreground` so `background` is painted behind it, stretched to
/// cover the same bounds. All interaction (clicks, tooltips, scrolling) is
/// forwarded only to `foreground` - `background` is purely decorative.
pub fn background_image<'a, Message, Theme, Renderer>(
    background: impl Into<Element<'a, Message, Theme, Renderer>>,
    foreground: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Element<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer + 'a,
    Message: 'a,
    Theme: 'a,
{
    Element::new(Backgrounded {
        background: background.into(),
        foreground: foreground.into(),
    })
}

struct Backgrounded<'a, Message, Theme, Renderer> {
    background: Element<'a, Message, Theme, Renderer>,
    foreground: Element<'a, Message, Theme, Renderer>,
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Backgrounded<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fill)
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.background), Tree::new(&self.foreground)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[&self.background, &self.foreground]);
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::stateless()
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &Limits,
    ) -> Node {
        let foreground_node =
            self.foreground
                .as_widget()
                .layout(&mut tree.children[1], renderer, limits);
        let size = foreground_node.size();

        let background_node = self.background.as_widget().layout(
            &mut tree.children[0],
            renderer,
            &Limits::new(size, size),
        );

        Node::with_children(size, vec![background_node, foreground_node])
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        let mut children = layout.children();
        let background_layout = children.next().expect("background layout");
        let foreground_layout = children.next().expect("foreground layout");

        self.background.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            background_layout,
            cursor,
            viewport,
        );
        self.foreground.as_widget().draw(
            &tree.children[1],
            renderer,
            theme,
            style,
            foreground_layout,
            cursor,
            viewport,
        );
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation<Message>,
    ) {
        let foreground_layout = layout
            .children()
            .nth(1)
            .expect("foreground layout");
        self.foreground.as_widget().operate(
            &mut tree.children[1],
            foreground_layout,
            renderer,
            operation,
        );
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &iced::Rectangle,
    ) -> event::Status {
        let foreground_layout = layout
            .children()
            .nth(1)
            .expect("foreground layout");
        self.foreground.as_widget_mut().on_event(
            &mut tree.children[1],
            event,
            foreground_layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        )
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &iced::Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let foreground_layout = layout
            .children()
            .nth(1)
            .expect("foreground layout");
        self.foreground.as_widget().mouse_interaction(
            &tree.children[1],
            foreground_layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        translation: iced::Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let foreground_layout = layout
            .children()
            .nth(1)
            .expect("foreground layout");
        self.foreground.as_widget_mut().overlay(
            &mut tree.children[1],
            foreground_layout,
            renderer,
            translation,
        )
    }
}
