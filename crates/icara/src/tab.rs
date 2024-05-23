//! A tab widget for displaying content with optional leading and trailing elements.
//!
//! The [`Tab`] widget allows you to create interactive tabs.
//! Optional leading and trailing elements can be used for icons and close buttons.
//!
//! This widget can be used inside a TabBar widget to support interactive reordering of tabs. TODO: implement this

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self};
use iced::advanced::widget::{self, tree, Tree, Widget};
use iced::widget::canvas::path::lyon_path::geom::euclid::Transform2D;
use iced::widget::canvas::{self, LineJoin, Stroke};
use iced::{event, mouse, touch, Event, Padding, Theme, Transformation};
use iced::{Color, Element, Length, Rectangle, Size};
use iced::{Point, Renderer};

/// The appearance of the tab
#[derive(Debug, Clone, Copy, Default)]
pub struct Appearance {}

/// A trait for defining the styling of the tab.
pub trait StyleSheet {
    /// The type of the style used for the tab.
    type Style: Default;
}

/// The default theme for the tab.
#[derive(Debug, Default)]
pub struct ThemeTab();

impl StyleSheet for Theme {
    type Style = ThemeTab;
}

/// A tab widget for displaying content with optional leading and trailing elements.
///
// TODO: add examples
pub struct Tab<'a, Message, Theme = iced::Theme>
where
    Theme: StyleSheet,
{
    content: Element<'a, Message, Theme, iced::Renderer>,
    content_leading: Option<Element<'a, Message, Theme, iced::Renderer>>,
    content_trailing: Option<Element<'a, Message, Theme, iced::Renderer>>,
    on_press: Option<Message>,
    width: Length,
    height: Length,
    padding: Padding,
}

impl<'a, Message, Theme> Tab<'a, Message, Theme>
where
    Theme: StyleSheet,
{
    /// Creates a new [`Tab`] with the given content.
    ///
    /// The tab's size will be determined by the size of the content.
    pub fn new(content: impl Into<Element<'a, Message, Theme>>) -> Self {
        let content = content.into();
        Self {
            content,
            content_leading: None,
            content_trailing: None,
            padding: Padding::new(5.0),
            on_press: None,
            width: Length::Shrink,
            height: Length::Shrink,
        }
    }

    /// Sets the width of the [`Tab`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Tab`].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the padding of the [`Tab`].
    pub fn padding(mut self, padding: Padding) -> Self {
        self.padding = padding;
        self
    }

    /// Sets the message that will be sent when the [`Tab`] is pressed.
    pub fn on_press(mut self, on_press: Message) -> Self {
        self.on_press = Some(on_press);
        self
    }

    /// Sets the message that will be sent when the [`Tab`] is pressed, if `Some`.
    /// If `None`, the [`Tab`] will not be interactive.
    pub fn on_press_maybe(mut self, on_press: Option<Message>) -> Self {
        self.on_press = on_press;
        self
    }

    /// Sets the leading element of the [`Tab`].
    pub fn leading(mut self, element: impl Into<Element<'a, Message, Theme>>) -> Self {
        self.content_leading = Some(element.into());
        self
    }

    /// Sets the leading element of the [`Tab`], if `Some`.
    /// If `None`, no leading element will be displayed.
    pub fn leading_maybe(mut self, element: Option<Element<'a, Message, Theme>>) -> Self {
        self.content_leading = element;
        self
    }

    /// Sets the trailing element of the [`Tab`].
    pub fn trailing(mut self, element: impl Into<Element<'a, Message, Theme>>) -> Self {
        self.content_trailing = Some(element.into());
        self
    }

    /// Sets the trailing element of the [`Tab`], if `Some`.
    /// If `None`, no trailing element will be displayed.
    pub fn trailing_maybe(mut self, element: Option<Element<'a, Message, Theme>>) -> Self {
        self.content_trailing = element;
        self
    }
}

/// The state of the tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct State {
    is_pressed: bool,
}

impl<'a, Message, Theme> Widget<Message, Theme, iced::Renderer> for Tab<'a, Message, Theme>
where
    Message: 'a + Clone,
    Theme: StyleSheet,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn children(&self) -> Vec<Tree> {
        let mut children = Vec::new();
        if let Some(content_leading) = &self.content_leading {
            children.push(Tree::new(content_leading));
        }
        children.push(Tree::new(&self.content));
        if let Some(content_trailing) = &self.content_trailing {
            children.push(Tree::new(content_trailing));
        }
        children
    }

    fn diff(&self, tree: &mut Tree) {
        let mut children = Vec::new();
        if let Some(content_leading) = &self.content_leading {
            children.push(content_leading);
        }
        children.push(&self.content);
        if let Some(content_trailing) = &self.content_trailing {
            children.push(content_trailing);
        }
        tree.diff_children(&children);
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(
        &self,
        tree: &mut widget::Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let mut children = Vec::new();
        let mut child_iter = tree.children.iter_mut();

        if let Some(content_leading) = &self.content_leading {
            let leading = content_leading.as_widget().layout(
                child_iter.next().unwrap(),
                renderer,
                &limits.loose(),
            );
            children.push(leading);
        }

        let content =
            self.content
                .as_widget()
                .layout(child_iter.next().unwrap(), renderer, &limits.loose());
        children.push(content);

        if let Some(content_trailing) = &self.content_trailing {
            let trailing = content_trailing.as_widget().layout(
                child_iter.next().unwrap(),
                renderer,
                &limits.loose(),
            );
            children.push(trailing);
        }

        // Use the children to determine the size of the entire tab
        let size = children.iter().fold(Size::ZERO, |acc, child| {
            Size::new(
                acc.width + child.bounds().width,
                acc.height.max(child.bounds().height),
            )
        });

        layout::Node::with_children(size, children)
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        use iced::advanced::Renderer;
        let radius = 10.0;
        let padding = 2.0;
        let bounds = layout.bounds();
        let mut frame = canvas::Frame::new(
            renderer,
            bounds
                .size()
                .expand(Size::new(padding * 2.0, padding * 2.0)),
        );
        let mut builder = canvas::path::Builder::new();
        let b = bounds;
        let corner_bl = Point::new(0.0, b.height);
        let corner_tl = Point::new(0.0, 0.0);
        let corner_tr = Point::new(b.width, 0.0);
        let corner_br = Point::new(b.width, b.height);

        builder.move_to(corner_bl);
        builder.arc_to(corner_tl, corner_tr, radius);
        builder.arc_to(corner_tr, corner_br, radius);
        builder.line_to(corner_br);

        let path = builder
            .build()
            .transform(&Transform2D::translation(padding, padding));
        frame.stroke(
            &path,
            Stroke {
                style: canvas::Style::Solid(Color::BLACK),
                width: 1.5,
                line_join: LineJoin::Round,
                ..Default::default()
            },
        );
        let state = tree.state.downcast_ref::<State>();
        if state.is_pressed {
            frame.fill(
                &path,
                canvas::Fill {
                    style: canvas::Style::Solid(Color::from_rgb8(100, 240, 240)),
                    ..canvas::Fill::default()
                },
            );
        }

        renderer.with_transformation(
            Transformation::translate(b.x - padding, b.y - padding),
            |renderer| {
                canvas::Renderer::draw(renderer, vec![frame.into_geometry()]);
            },
        );

        let mut children = layout.children().zip(tree.children.iter());

        // TODO: correctly order of leading, content, trailing

        if let Some(content_leading) = &self.content_leading {
            let (child_layout, child_state) = children.next().unwrap();
            content_leading.as_widget().draw(
                child_state,
                renderer,
                theme,
                &renderer::Style {
                    text_color: style.text_color,
                },
                child_layout,
                cursor,
                viewport,
            );
        }

        let (child_layout, child_state) = children.next().unwrap();
        self.content.as_widget().draw(
            child_state,
            renderer,
            theme,
            &renderer::Style {
                text_color: style.text_color,
            },
            child_layout,
            cursor,
            viewport,
        );

        if let Some(content_trailing) = &self.content_trailing {
            let (child_layout, child_state) = children.next().unwrap();
            content_trailing.as_widget().draw(
                child_state,
                renderer,
                theme,
                &renderer::Style {
                    text_color: style.text_color,
                },
                child_layout,
                cursor,
                viewport,
            );
        }
    }

    fn operate(
        &self,
        state: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn widget::Operation<Message>,
    ) {
        // TODO: implement this for content_leading and content_trailing
        operation.container(None, layout.bounds(), &mut |operation| {
            self.content.as_widget().operate(
                &mut state.children[0],
                layout.children().next().unwrap(),
                renderer,
                operation,
            );
        });
    }

    fn on_event(
        &mut self,
        state: &mut Tree,
        event: iced::Event,
        layout: Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> iced::advanced::graphics::core::event::Status {
        let mut children = state.children.iter_mut().zip(layout.children());

        if let Some(content_leading) = &mut self.content_leading {
            let (child, layout) = children.next().unwrap();
            if let event::Status::Captured = content_leading.as_widget_mut().on_event(
                child,
                event.clone(),
                layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            ) {
                return event::Status::Captured;
            }
        }

        if let Some((child, layout)) = children.next() {
            if let event::Status::Captured = self.content.as_widget_mut().on_event(
                child,
                event.clone(),
                layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            ) {
                return event::Status::Captured;
            }
        }

        if let Some(content_trailing) = &mut self.content_trailing {
            let (child, layout) = children.next().unwrap();
            if let event::Status::Captured = content_trailing.as_widget_mut().on_event(
                child,
                event.clone(),
                layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            ) {
                return event::Status::Captured;
            }
        }

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if self.on_press.is_some() {
                    let bounds = layout.bounds();

                    if cursor.is_over(bounds) {
                        let state = state.state.downcast_mut::<State>();

                        state.is_pressed = true;

                        return event::Status::Captured;
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. }) => {
                if let Some(on_press) = self.on_press.clone() {
                    let state = state.state.downcast_mut::<State>();

                    if state.is_pressed {
                        state.is_pressed = false;

                        let bounds = layout.bounds();

                        if cursor.is_over(bounds) {
                            shell.publish(on_press);
                        }

                        return event::Status::Captured;
                    }
                }
            }
            Event::Touch(touch::Event::FingerLost { .. }) => {
                let state = state.state.downcast_mut::<State>();

                state.is_pressed = false;
            }
            _ => {}
        }

        event::Status::Ignored
    }

    fn mouse_interaction(
        &self,
        _state: &Tree,
        layout: Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &iced::Renderer,
    ) -> iced::advanced::mouse::Interaction {
        let is_mouse_over = cursor.is_over(layout.bounds());

        if is_mouse_over && self.on_press.is_some() {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }

    fn overlay<'b>(
        &'b mut self,
        state: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        translation: iced::Vector,
    ) -> Option<iced::advanced::overlay::Element<'b, Message, Theme, iced::Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut state.children[0],
            layout.children().next().unwrap(),
            renderer,
            translation,
        )
    }
}

impl<'a, Message, Theme> From<Tab<'a, Message, Theme>> for Element<'a, Message, Theme>
where
    Message: Clone + 'a,
    Theme: StyleSheet + 'a,
    Renderer: renderer::Renderer + 'a,
{
    fn from(tab: Tab<'a, Message, Theme>) -> Self {
        Self::new(tab)
    }
}
