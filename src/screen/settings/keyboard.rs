//! Keyboard support local to Settings. The pinned Iced buttons and checkboxes
//! do not register focus or keyboard activation themselves.
use iced::advanced::widget::{self, Operation, operation};
use iced::advanced::{Layout, Shell, renderer};
use iced::{
    Border, Color, Event, Length, Padding, Rectangle, Size, Task, Vector,
    keyboard, mouse,
};

use crate::Theme;
use crate::widget::{Element, Renderer, decorate};

pub(super) mod picker;
pub(super) use picker::pick_list;

pub(super) const ROOT: &str = "settings-keyboard-root";

#[derive(Debug)]
pub(super) struct Focus {
    id: widget::Id,
    focused: bool,
}
impl Default for Focus {
    fn default() -> Self {
        Self {
            id: widget::Id::unique(),
            focused: false,
        }
    }
}
impl operation::Focusable for Focus {
    fn is_focused(&self) -> bool {
        self.focused
    }
    fn focus(&mut self) {
        self.focused = true;
    }
    fn unfocus(&mut self) {
        self.focused = false;
    }
}

fn focusable<'a, M: Clone + 'a>(
    inner: Element<'a, M>,
    activation: Option<M>,
    id: Option<widget::Id>,
) -> Element<'a, M> {
    let enabled = activation.is_some();
    decorate(inner)
        .operate(
            move |state: &mut Focus,
                  inner: &mut Element<'a, M>,
                  tree: &mut widget::Tree,
                  layout: Layout,
                  viewport: &Rectangle,
                  renderer: &Renderer,
                  op: &mut dyn Operation| {
                if let Some(id) = &id {
                    state.id = id.clone();
                }
                if enabled {
                    op.focusable(
                        Some(&state.id.clone()),
                        layout.bounds(),
                        state,
                    );
                }
                inner
                    .as_widget_mut()
                    .operate(tree, layout, viewport, renderer, op);
            },
        )
        .update(
            move |state: &mut Focus,
                  inner: &mut Element<'a, M>,
                  tree: &mut widget::Tree,
                  event: &Event,
                  layout: Layout,
                  cursor: mouse::Cursor,
                  renderer: &Renderer,
                  shell: &mut Shell<'_, M>,
                  viewport: &Rectangle| {
                if let Event::Mouse(mouse::Event::ButtonPressed {
                    button: mouse::Button::Left,
                    ..
                }) = event
                {
                    state.focused = enabled
                        && cursor.is_over(layout.bounds())
                        && cursor.is_over(*viewport);
                    shell.request_redraw();
                }
                if state.focused
                    && !shell.is_event_captured()
                    && let Event::Keyboard(keyboard::Event::KeyPressed {
                        key,
                        modifiers,
                        ..
                    }) = event
                    && !modifiers.command()
                    && !modifiers.alt()
                    && matches!(
                        key.as_ref(),
                        keyboard::Key::Named(
                            keyboard::key::Named::Enter
                                | keyboard::key::Named::Space
                        )
                    )
                    && let Some(message) = &activation
                {
                    shell.publish(message.clone());
                    shell.capture_event();
                    return;
                }
                inner.as_widget_mut().update(
                    tree, event, layout, cursor, renderer, shell, viewport,
                );
            },
        )
        .draw(
            |state: &Focus,
             inner: &Element<'a, M>,
             tree: &widget::Tree,
             renderer: &mut Renderer,
             theme: &Theme,
             style: &renderer::Style,
             layout: Layout,
             cursor: mouse::Cursor,
             viewport: &Rectangle| {
                inner.as_widget().draw(
                    tree, renderer, theme, style, layout, cursor, viewport,
                );
                if state.focused {
                    draw_focus(renderer, theme, layout.bounds(), viewport);
                }
            },
        )
        .into()
}

pub(super) fn draw_focus(
    renderer: &mut Renderer,
    theme: &Theme,
    bounds: Rectangle,
    viewport: &Rectangle,
) {
    use iced::advanced::Renderer as _;
    if bounds.intersects(viewport) {
        renderer.with_layer(*viewport, |renderer| {
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: Border {
                        width: 2.0,
                        color: super::style::input(
                            theme,
                            iced::widget::text_input::Status::Focused {
                                is_hovered: false,
                            },
                        )
                        .border
                        .color,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                },
                Color::TRANSPARENT,
            );
        });
    }
}

pub(super) struct Button<'a, M> {
    inner: iced::widget::Button<'a, M, Theme>,
    activation: Option<M>,
    id: Option<widget::Id>,
}
pub(super) fn button<'a, M: Clone + 'a>(
    content: impl Into<Element<'a, M>>,
) -> Button<'a, M> {
    Button {
        inner: iced::widget::button(content),
        activation: None,
        id: None,
    }
}
impl<'a, M: Clone + 'a> Button<'a, M> {
    pub fn id(mut self, id: impl Into<widget::Id>) -> Self {
        self.id = Some(id.into());
        self
    }
    pub fn on_press(mut self, message: M) -> Self {
        self.inner = self.inner.on_press(message.clone());
        self.activation = Some(message);
        self
    }
    pub fn on_press_maybe(mut self, message: Option<M>) -> Self {
        self.inner = self.inner.on_press_maybe(message.clone());
        self.activation = message;
        self
    }
    pub fn padding(mut self, value: impl Into<Padding>) -> Self {
        self.inner = self.inner.padding(value);
        self
    }
    pub fn width(mut self, value: impl Into<Length>) -> Self {
        self.inner = self.inner.width(value);
        self
    }
    pub fn style(
        mut self,
        value: impl Fn(
            &Theme,
            iced::widget::button::Status,
        ) -> iced::widget::button::Style
        + 'a,
    ) -> Self {
        self.inner = self.inner.style(value);
        self
    }
}
impl<'a, M: Clone + 'a> From<Button<'a, M>> for Element<'a, M> {
    fn from(value: Button<'a, M>) -> Self {
        focusable(value.inner.into(), value.activation, value.id)
    }
}

pub(super) struct Checkbox<'a, M> {
    inner: iced::widget::Checkbox<'a, M, Theme>,
    checked: bool,
    activation: Option<M>,
    id: Option<widget::Id>,
}
pub(super) fn checkbox<'a, M: Clone + 'a>(checked: bool) -> Checkbox<'a, M> {
    Checkbox {
        inner: iced::widget::checkbox(checked),
        checked,
        activation: None,
        id: None,
    }
}
impl<'a, M: Clone + 'a> Checkbox<'a, M> {
    pub fn id(mut self, id: impl Into<widget::Id>) -> Self {
        self.id = Some(id.into());
        self
    }
    pub fn label(
        mut self,
        label: impl iced::widget::text::IntoFragment<'a>,
    ) -> Self {
        self.inner = self.inner.label(label);
        self
    }
    pub fn on_toggle(mut self, handler: impl Fn(bool) -> M + 'a) -> Self {
        self.activation = Some(handler(!self.checked));
        self.inner = self.inner.on_toggle(handler);
        self
    }
    pub fn style(
        mut self,
        style: impl Fn(
            &Theme,
            iced::widget::checkbox::Status,
        ) -> iced::widget::checkbox::Style
        + 'a,
    ) -> Self {
        self.inner = self.inner.style(style);
        self
    }
}
impl<'a, M: Clone + 'a> From<Checkbox<'a, M>> for Element<'a, M> {
    fn from(value: Checkbox<'a, M>) -> Self {
        focusable(value.inner.into(), value.activation, value.id)
    }
}

/// Keep inline confirmations within their active controls.
pub(super) fn inactive<'a, M: 'a>(
    inner: impl Into<Element<'a, M>>,
) -> Element<'a, M> {
    decorate(inner)
        .operate(
            |(): &mut (),
             _: &mut Element<'a, M>,
             _: &mut widget::Tree,
             _: Layout,
             _: &Rectangle,
             _: &Renderer,
             _: &mut dyn Operation| {},
        )
        .update(
            |(): &mut (),
             _: &mut Element<'a, M>,
             _: &mut widget::Tree,
             _: &Event,
             _: Layout,
             _: mouse::Cursor,
             _: &Renderer,
             _: &mut Shell<'_, M>,
             _: &Rectangle| {},
        )
        .into()
}

/// Children get first refusal, so Escape closes a picker before the window.
pub(super) fn shortcuts<'a, M: Clone + 'a>(
    inner: impl Into<Element<'a, M>>,
    handler: impl Fn(&keyboard::Key, keyboard::Modifiers) -> Option<M> + 'a,
) -> Element<'a, M> {
    decorate(inner)
        .update(
            move |_state: &mut (),
                  inner: &mut Element<'a, M>,
                  tree: &mut widget::Tree,
                  event: &Event,
                  layout: Layout,
                  cursor: mouse::Cursor,
                  renderer: &Renderer,
                  shell: &mut Shell<'_, M>,
                  viewport: &Rectangle| {
                inner.as_widget_mut().update(
                    tree, event, layout, cursor, renderer, shell, viewport,
                );
                if !shell.is_event_captured()
                    && let Event::Keyboard(keyboard::Event::KeyPressed {
                        key,
                        modifiers,
                        ..
                    }) = event
                    && let Some(message) = handler(key, *modifiers)
                {
                    shell.publish(message);
                    shell.capture_event();
                }
            },
        )
        .into()
}

#[derive(Clone)]
struct Scroll {
    id: widget::Id,
    bounds: Rectangle,
    content: Size,
    translation: Vector,
}
#[derive(Clone)]
struct Entry {
    id: Option<widget::Id>,
    focused: bool,
    bounds: Rectangle,
    scrolls: Vec<Scroll>,
}
#[derive(Clone)]
enum Target {
    Step(bool),
    Id(widget::Id),
}
struct Collect {
    target: Target,
    entries: Vec<Entry>,
    scrolls: Vec<Scroll>,
    pending: Option<Scroll>,
}
impl Operation for Collect {
    fn traverse(&mut self, visit: &mut dyn FnMut(&mut dyn Operation)) {
        let pushed = self.pending.take();
        if let Some(scroll) = &pushed {
            self.scrolls.push(scroll.clone());
        }
        visit(self);
        if pushed.is_some() {
            self.scrolls.pop();
        }
    }
    fn scrollable(
        &mut self,
        id: Option<&widget::Id>,
        bounds: Rectangle,
        content: Size,
        translation: Vector,
        _state: &mut dyn operation::Scrollable,
    ) {
        self.pending = id.map(|id| Scroll {
            id: id.clone(),
            bounds,
            content,
            translation,
        });
    }
    fn focusable(
        &mut self,
        id: Option<&widget::Id>,
        bounds: Rectangle,
        state: &mut dyn operation::Focusable,
    ) {
        self.entries.push(Entry {
            id: id.cloned(),
            focused: state.is_focused(),
            bounds,
            scrolls: self.scrolls.clone(),
        });
    }
    fn finish(&self) -> operation::Outcome<()> {
        let index = match &self.target {
            Target::Step(previous) => next_index(
                self.entries.len(),
                self.entries.iter().position(|e| e.focused),
                *previous,
            ),
            Target::Id(id) => {
                self.entries.iter().position(|e| e.id.as_ref() == Some(id))
            }
        };
        let Some(index) = index else {
            return operation::Outcome::None;
        };
        operation::Outcome::Chain(Box::new(Activate {
            index,
            current: 0,
            entry: self.entries[index].clone(),
        }))
    }
}
struct Activate {
    index: usize,
    current: usize,
    entry: Entry,
}
impl Operation for Activate {
    fn traverse(&mut self, visit: &mut dyn FnMut(&mut dyn Operation)) {
        visit(self);
    }
    fn focusable(
        &mut self,
        _id: Option<&widget::Id>,
        _bounds: Rectangle,
        state: &mut dyn operation::Focusable,
    ) {
        if self.current == self.index {
            state.focus();
        } else {
            state.unfocus();
        }
        self.current += 1;
    }
    fn scrollable(
        &mut self,
        id: Option<&widget::Id>,
        _bounds: Rectangle,
        _content: Size,
        _translation: Vector,
        state: &mut dyn operation::Scrollable,
    ) {
        if let Some(scroll) =
            self.entry.scrolls.iter().find(|s| Some(&s.id) == id)
        {
            let y = reveal_offset(
                self.entry.bounds,
                scroll.bounds,
                scroll.translation.y,
            );
            state.scroll_to(
                operation::scrollable::AbsoluteOffset {
                    x: None,
                    y: Some(y),
                },
                operation::Animation::Instant,
                scroll.bounds,
                scroll.content,
            );
        }
    }
}
fn next_index(
    total: usize,
    current: Option<usize>,
    previous: bool,
) -> Option<usize> {
    if total == 0 {
        None
    } else {
        Some(match (current, previous) {
            (None, false) => 0,
            (None, true) => total - 1,
            (Some(i), false) => (i + 1) % total,
            (Some(i), true) => (i + total - 1) % total,
        })
    }
}
fn reveal_offset(target: Rectangle, viewport: Rectangle, offset: f32) -> f32 {
    let top = viewport.y + offset + 4.0;
    let bottom = viewport.y + offset + viewport.height - 4.0;
    if target.y < top {
        (target.y - viewport.y - 4.0).max(0.0)
    } else if target.y + target.height > bottom {
        (target.y + target.height - viewport.y - viewport.height + 4.0).max(0.0)
    } else {
        offset
    }
}
fn traverse<M: Send + 'static>(target: Target) -> Task<M> {
    widget::operate(operation::scope(
        widget::Id::new(ROOT),
        Collect {
            target,
            entries: vec![],
            scrolls: vec![],
            pending: None,
        },
    ))
    .discard()
}
pub(super) fn step<M: Send + 'static>(previous: bool) -> Task<M> {
    traverse(Target::Step(previous))
}
pub(super) fn focus<M: Send + 'static>(id: impl Into<widget::Id>) -> Task<M> {
    traverse(Target::Id(id.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn traversal_wraps_in_both_directions_without_losing_focus() {
        assert_eq!(next_index(3, Some(2), false), Some(0));
        assert_eq!(next_index(3, Some(0), true), Some(2));
        assert_eq!(next_index(3, None, true), Some(2));
        assert_eq!(next_index(0, None, false), None);
    }
    #[test]
    fn reveal_keeps_visible_fields_and_scrolls_both_edges() {
        let viewport = Rectangle {
            x: 0.0,
            y: 50.0,
            width: 300.0,
            height: 200.0,
        };
        let target = |y| Rectangle {
            x: 0.0,
            y,
            width: 150.0,
            height: 30.0,
        };
        assert_eq!(reveal_offset(target(100.0), viewport, 0.0), 0.0);
        assert_eq!(reveal_offset(target(270.0), viewport, 0.0), 54.0);
        assert_eq!(reveal_offset(target(70.0), viewport, 100.0), 16.0);
    }
}
