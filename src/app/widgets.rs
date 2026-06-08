use iced::advanced::layout;
use iced::advanced::overlay;
use iced::advanced::renderer;
use iced::advanced::widget::tree::{self, Tree};
use iced::advanced::{Clipboard, Layout, Shell, Widget};
use iced::{Element, Event, Length, Rectangle, Renderer, Size, Theme, Vector, mouse};

use crate::app::messages::Message;

pub(crate) fn slider_with_handle_cursor<'a>(
    slider: impl Into<Element<'a, Message>>,
    value: f64,
    range: std::ops::RangeInclusive<f64>,
    handle_radius: f32,
) -> Element<'a, Message> {
    Element::new(SliderHandleCursor {
        slider: slider.into(),
        value,
        range,
        handle_radius,
    })
}

struct SliderHandleCursor<'a> {
    slider: Element<'a, Message>,
    value: f64,
    range: std::ops::RangeInclusive<f64>,
    handle_radius: f32,
}

impl Widget<Message, Theme, Renderer> for SliderHandleCursor<'_> {
    fn size(&self) -> Size<Length> {
        self.slider.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.slider.as_widget().size_hint()
    }

    fn tag(&self) -> tree::Tag {
        self.slider.as_widget().tag()
    }

    fn state(&self) -> tree::State {
        self.slider.as_widget().state()
    }

    fn children(&self) -> Vec<Tree> {
        self.slider.as_widget().children()
    }

    fn diff(&self, tree: &mut Tree) {
        self.slider.as_widget().diff(tree);
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.slider.as_widget_mut().layout(tree, renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn iced::advanced::widget::Operation,
    ) {
        self.slider
            .as_widget_mut()
            .operate(tree, layout, renderer, operation);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.slider.as_widget_mut().update(
            tree, event, layout, cursor, renderer, clipboard, shell, viewport,
        );
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.slider
            .as_widget()
            .draw(tree, renderer, theme, style, layout, cursor, viewport);
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let slider_interaction = self
            .slider
            .as_widget()
            .mouse_interaction(tree, layout, cursor, viewport, renderer);

        if slider_interaction == mouse::Interaction::Grabbing {
            return slider_interaction;
        }

        if cursor.is_over(self.handle_bounds(layout.bounds())) {
            return slider_interaction;
        }

        if cursor.is_over(layout.bounds()) {
            return mouse::Interaction::Pointer;
        }

        mouse::Interaction::default()
    }

    fn overlay<'a>(
        &'a mut self,
        tree: &'a mut Tree,
        layout: Layout<'a>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'a, Message, Theme, Renderer>> {
        self.slider
            .as_widget_mut()
            .overlay(tree, layout, renderer, viewport, translation)
    }
}

impl SliderHandleCursor<'_> {
    fn handle_bounds(&self, bounds: Rectangle) -> Rectangle {
        let (range_start, range_end) = self.range.clone().into_inner();
        let handle_width = self.handle_radius * 2.0;
        let offset = if range_start >= range_end {
            0.0
        } else {
            (f64::from(bounds.width - handle_width) * (self.value - range_start)
                / (range_end - range_start)) as f32
        };

        Rectangle {
            x: bounds.x + offset,
            y: bounds.y + bounds.height / 2.0 - self.handle_radius,
            width: handle_width,
            height: handle_width,
        }
    }
}
