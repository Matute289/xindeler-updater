//! A widget that paints an image behind another element, filling the same
//! bounds. Needed because this iced version (0.12) has no `Stack` widget and
//! `container::Appearance` only supports solid colors/gradients as a
//! background, not images.
use iced::{
    ContentFit, Element, Length, Rectangle, Size, Vector,
    advanced::{
        Clipboard, Layout, Shell, Widget,
        image::{self, Handle},
        layout::{Limits, Node},
        mouse, overlay, renderer,
        widget::{Operation, Tree, tree},
    },
    event::{self, Event},
};

/// A `Cover`-fit image, cropped centered on overflow in both dimensions.
///
/// `iced_widget::image::Image`'s own `ContentFit::Cover` only centers the
/// dimension that fits; the overflowing dimension is anchored to the
/// top-left and the overflow past the bottom/right edge gets clipped away
/// (see `iced_widget::image::draw`'s `offset` calculation, which `.max(0.0)`s
/// away any negative/centering offset). For a background image that's a
/// systematic bias - whatever's centered in the source photo ends up pushed
/// toward the bottom-right of what's visible. This widget fixes that by
/// always centering the crop.
pub fn centered_cover<'a, Message, Theme, Renderer>(
    handle: Handle,
) -> Element<'a, Message, Theme, Renderer>
where
    Renderer: image::Renderer<Handle = Handle> + 'a,
    Message: 'a,
    Theme: 'a,
{
    Element::new(CoveredImage { handle })
}

struct CoveredImage {
    handle: Handle,
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for CoveredImage
where
    Renderer: image::Renderer<Handle = Handle>,
{
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fill)
    }

    fn layout(&self, _tree: &mut Tree, _renderer: &Renderer, limits: &Limits) -> Node {
        Node::new(limits.resolve(Length::Fill, Length::Fill, Size::ZERO))
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let image_size = {
            let Size { width, height } = renderer.dimensions(&self.handle);
            Size::new(width as f32, height as f32)
        };
        let adjusted_fit = ContentFit::Cover.fit(image_size, bounds.size());

        let offset = Vector::new(
            (bounds.width - adjusted_fit.width) / 2.0,
            (bounds.height - adjusted_fit.height) / 2.0,
        );
        let drawing_bounds = Rectangle {
            width: adjusted_fit.width,
            height: adjusted_fit.height,
            ..bounds
        };

        renderer.with_layer(bounds, |renderer| {
            renderer.draw(
                self.handle.clone(),
                image::FilterMethod::default(),
                drawing_bounds + offset,
            );
        });
    }
}

/// Stacks `foreground` on top of `background`, both filling the same bounds.
/// All interaction (clicks, tooltips, scrolling) is forwarded only to
/// `foreground` - `background` is rendered but never receives events, which
/// is what makes this useful both for a decorative background photo and for
/// a modal dialog that should block interaction with whatever's behind it.
pub fn layered<'a, Message, Theme, Renderer>(
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

    fn layout(&self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
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
        let bounds = layout.bounds();

        // iced_wgpu batches primitives by kind (quads, then images, then text) rather
        // than strictly respecting draw() call order, so without explicit layers the
        // background image - drawn first, but still an "image" primitive - ends up
        // painted over every quad/text primitive in the whole frame instead of just
        // behind `foreground`. Wrapping each half in its own layer forces it to
        // composite as a single, correctly self-ordered unit before the two are
        // stacked in draw order.
        renderer.with_layer(bounds, |renderer| {
            self.background.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                style,
                background_layout,
                cursor,
                viewport,
            );
        });
        renderer.with_layer(bounds, |renderer| {
            self.foreground.as_widget().draw(
                &tree.children[1],
                renderer,
                theme,
                style,
                foreground_layout,
                cursor,
                viewport,
            );
        });
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation<Message>,
    ) {
        let foreground_layout = layout.children().nth(1).expect("foreground layout");
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
        let foreground_layout = layout.children().nth(1).expect("foreground layout");
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
        let foreground_layout = layout.children().nth(1).expect("foreground layout");
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
        let foreground_layout = layout.children().nth(1).expect("foreground layout");
        self.foreground.as_widget_mut().overlay(
            &mut tree.children[1],
            foreground_layout,
            renderer,
            translation,
        )
    }
}
