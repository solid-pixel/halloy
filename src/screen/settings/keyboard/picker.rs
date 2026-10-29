//! Settings-local adaptation of Iced's pick list, with keyboard focus and selection.
//! Based on iced_widget/src/pick_list.rs at 138b1d0.
//! Copyright 2019 Héctor Ramón, Iced contributors. See ICED-LICENSE.
//! Kept local because the rest of Halloy uses the upstream widget unchanged.
use std::borrow::Borrow;
use std::f32;

use iced::advanced::text::{self, Text, paragraph};
use iced::advanced::widget::tree::{self, Tree};
use iced::advanced::widget::{self};
use iced::advanced::{Layout, Shell, Widget, layout, overlay, renderer};
use iced::widget::overlay::menu::{self, Menu};
use iced::widget::pick_list::{Catalog, Handle, Icon, Status, Style, StyleFn};
use iced::{
    Element, Event, Font, Length, Padding, Pixels, Point, Rectangle, Size,
    Vector, alignment, keyboard, mouse, touch, window,
};

pub(crate) fn pick_list<'a, T: PartialEq + Clone + 'a, M: Clone + 'a>(
    selected: Option<T>,
    options: Vec<T>,
    to_string: impl Fn(&T) -> String + 'a,
) -> PickList<'a, T, Vec<T>, T, M, crate::Theme> {
    PickList::new(selected, options, to_string)
}

pub struct PickList<'a, T, L, V, Message, Theme = crate::Theme>
where
    T: PartialEq + Clone,
    L: Borrow<[T]> + 'a,
    V: Borrow<T> + 'a,
    Theme: Catalog,
{
    id: Option<widget::Id>,
    options: L,
    to_string: Box<dyn Fn(&T) -> String + 'a>,
    on_select: Option<Box<dyn Fn(T) -> Message + 'a>>,
    on_open: Option<Message>,
    on_close: Option<Message>,
    placeholder: Option<String>,
    selected: Option<V>,
    width: Length,
    padding: Padding,
    text_size: Option<Pixels>,
    line_height: Option<text::LineHeight>,
    shaping: text::Shaping,
    ellipsis: text::Ellipsis,
    font: Option<Font>,
    handle: Handle,
    class: <Theme as Catalog>::Class<'a>,
    menu_class: <Theme as menu::Catalog>::Class<'a>,
    last_status: Option<Status>,
    menu_height: Length,
}

impl<'a, T, L, V, Message, Theme> PickList<'a, T, L, V, Message, Theme>
where
    T: PartialEq + Clone,
    L: Borrow<[T]> + 'a,
    V: Borrow<T> + 'a,
    Message: Clone,
    Theme: Catalog,
{
    pub fn new(
        selected: Option<V>,
        options: L,
        to_string: impl Fn(&T) -> String + 'a,
    ) -> Self {
        Self {
            id: None,
            to_string: Box::new(to_string),
            on_select: None,
            on_open: None,
            on_close: None,
            options,
            placeholder: None,
            selected,
            width: Length::Shrink,
            padding: Padding::new(6.0),
            text_size: None,
            line_height: Some(crate::font::current_line_height()),
            shaping: text::Shaping::default(),
            ellipsis: text::Ellipsis::End,
            font: None,
            handle: Handle::default(),
            class: <Theme as Catalog>::default(),
            menu_class: <Theme as Catalog>::default_menu(),
            last_status: None,
            menu_height: Length::Shrink,
        }
    }

    pub fn id(mut self, id: impl Into<widget::Id>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    pub fn on_select(mut self, on_select: impl Fn(T) -> Message + 'a) -> Self {
        self.on_select = Some(Box::new(on_select));
        self
    }

    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme, Status) -> Style + 'a) -> Self
    where
        <Theme as Catalog>::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }

    #[must_use]
    pub fn menu_style(
        mut self,
        style: impl Fn(&Theme) -> menu::Style + 'a,
    ) -> Self
    where
        <Theme as menu::Catalog>::Class<'a>: From<menu::StyleFn<'a, Theme>>,
    {
        self.menu_class = (Box::new(style) as menu::StyleFn<'a, Theme>).into();
        self
    }
}

impl<'a, T, L, V, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for PickList<'a, T, L, V, Message, Theme>
where
    T: Clone + PartialEq + 'a,
    L: Borrow<[T]>,
    V: Borrow<T>,
    Message: Clone + 'a,
    Theme: Catalog + 'a,
    Renderer: text::Renderer + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State<Renderer::Paragraph>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::<Renderer::Paragraph>::new())
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: Length::Shrink,
        }
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) {
        let state = tree.state.downcast_mut::<State<Renderer::Paragraph>>();

        let font = self.font.unwrap_or_else(|| renderer.font());
        let text_size = self.text_size.unwrap_or_else(|| renderer.text_size());
        let line_height =
            self.line_height.unwrap_or_else(|| renderer.line_height());
        let options = self.options.borrow();

        let option_text = Text {
            content: "",
            bounds: Size::new(
                limits.bounds().width,
                line_height.to_absolute(text_size).into(),
            ),
            size: text_size,
            line_height,
            font,
            align_x: text::Alignment::Default,
            align_y: alignment::Vertical::Center,
            shaping: self.shaping,
            wrapping: text::Wrapping::None,
            ellipsis: self.ellipsis,
            hint_factor: renderer.hint_factor(),
        };

        if let Some(placeholder) = &self.placeholder {
            let _ = state.placeholder.update(Text {
                content: placeholder,
                ..option_text
            });
        }

        let max_width = match self.width {
            Length::Shrink => {
                state.options.resize_with(options.len(), Default::default);

                for (option, paragraph) in
                    options.iter().zip(state.options.iter_mut())
                {
                    let label = (self.to_string)(option);

                    let _ = paragraph.update(Text {
                        content: &label,
                        ..option_text
                    });
                }

                let labels_width =
                    state.options.iter().fold(0.0, |width, paragraph| {
                        f32::max(width, paragraph.min_width())
                    });

                labels_width.max(
                    self.placeholder
                        .as_ref()
                        .map_or(0.0, |_| state.placeholder.min_width()),
                )
            }
            _ => 0.0,
        };

        let size = {
            let intrinsic = Size::new(
                max_width + text_size.0 + self.padding.left,
                f32::from(line_height.to_absolute(text_size)),
            );

            limits
                .width(self.width)
                .shrink(self.padding)
                .resolve(self.width, Length::Shrink, intrinsic)
                .expand(self.padding)
        };

        tree.size = size;
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State<Renderer::Paragraph>>();

        // Every control receives pointer focus changes, including when an
        // earlier sibling already handled the click.
        if matches!(
            event,
            Event::Mouse(mouse::Event::ButtonPressed {
                button: mouse::Button::Left,
                ..
            }) | Event::Touch(touch::Event::FingerPressed { .. })
        ) {
            state.focus.focused = self.on_select.is_some()
                && cursor.is_over(layout.bounds())
                && cursor.is_over(*_viewport);
            if !state.focus.focused {
                state.is_open = false;
            }
        }
        if shell.is_event_captured() {
            return;
        }
        if let Event::Keyboard(keyboard::Event::KeyPressed {
            key,
            modifiers,
            ..
        }) = event
            && state.focus.focused
            && self.on_select.is_some()
            && !modifiers.command()
            && !modifiers.alt()
        {
            use keyboard::key::Named;
            let options = self.options.borrow();
            let selected = self.selected.as_ref().map(Borrow::borrow);
            let selected_index =
                options.iter().position(|option| Some(option) == selected);
            match key.as_ref() {
                keyboard::Key::Named(Named::Escape) if state.is_open => {
                    state.is_open = false;
                    shell.capture_event();
                    shell.request_redraw();
                    return;
                }
                keyboard::Key::Named(Named::Tab) => {
                    state.is_open = false;
                    shell.request_redraw();
                }
                keyboard::Key::Named(Named::Enter | Named::Space) => {
                    if state.is_open {
                        if let Some(option) =
                            state.hovered_option.and_then(|i| options.get(i))
                        {
                            shell.publish(self.on_select.as_ref().unwrap()(
                                option.clone(),
                            ));
                        }
                        state.is_open = false;
                    } else {
                        state.is_open = true;
                        state.hovered_option = selected_index
                            .or_else(|| (!options.is_empty()).then_some(0));
                        state.reveal = true;
                    }
                    shell.capture_event();
                    shell.request_redraw();
                    return;
                }
                keyboard::Key::Named(
                    named @ (Named::ArrowDown
                    | Named::ArrowUp
                    | Named::Home
                    | Named::End),
                ) => {
                    state.hovered_option = move_option(
                        options.len(),
                        if state.is_open {
                            state.hovered_option
                        } else {
                            selected_index
                        },
                        named,
                    );
                    state.is_open = true;
                    state.reveal = true;
                    shell.capture_event();
                    shell.request_redraw();
                    return;
                }
                keyboard::Key::Character(prefix) if state.is_open => {
                    let prefix = prefix.to_lowercase();
                    let start = state.hovered_option.map_or(0, |i| i + 1);
                    state.hovered_option = (start..options.len())
                        .chain(0..start.min(options.len()))
                        .find(|&i| {
                            (self.to_string)(&options[i])
                                .to_lowercase()
                                .starts_with(&prefix)
                        })
                        .or(state.hovered_option);
                    state.reveal = true;
                    shell.capture_event();
                    shell.request_redraw();
                    return;
                }
                _ => {}
            }
        }
        match event {
            Event::Mouse(mouse::Event::ButtonPressed {
                button: mouse::Button::Left,
                ..
            })
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                state.focus.focused = self.on_select.is_some()
                    && cursor.is_over(layout.bounds())
                    && cursor.is_over(*_viewport);
                if state.is_open {
                    // Event wasn't processed by overlay, so cursor was clicked either outside its
                    // bounds or on the drop-down, either way we close the overlay.
                    state.is_open = false;

                    if let Some(on_close) = &self.on_close {
                        shell.publish(on_close.clone());
                    }

                    shell.capture_event();
                } else if self.on_select.is_some()
                    && cursor.is_over(layout.bounds())
                {
                    let selected = self.selected.as_ref().map(Borrow::borrow);

                    state.is_open = true;
                    state.reveal = true;
                    state.hovered_option = self
                        .options
                        .borrow()
                        .iter()
                        .position(|option| Some(option) == selected);

                    if let Some(on_open) = &self.on_open {
                        shell.publish(on_open.clone());
                    }

                    shell.capture_event();
                }
            }
            Event::Mouse(mouse::Event::WheelScrolled {
                delta: mouse::ScrollDelta::Lines { y, .. },
            }) => {
                let Some(on_select) = &self.on_select else {
                    return;
                };

                if state.keyboard_modifiers.command()
                    && cursor.is_over(layout.bounds())
                    && !state.is_open
                {
                    fn find_next<'a, T: PartialEq>(
                        selected: &'a T,
                        mut options: impl Iterator<Item = &'a T>,
                    ) -> Option<&'a T> {
                        let _ = options.find(|&option| option == selected);

                        options.next()
                    }

                    let options = self.options.borrow();
                    let selected = self.selected.as_ref().map(Borrow::borrow);

                    let next_option = if *y < 0.0 {
                        if let Some(selected) = selected {
                            find_next(selected, options.iter())
                        } else {
                            options.first()
                        }
                    } else if *y > 0.0 {
                        if let Some(selected) = selected {
                            find_next(selected, options.iter().rev())
                        } else {
                            options.last()
                        }
                    } else {
                        None
                    };

                    if let Some(next_option) = next_option {
                        shell.publish(on_select(next_option.clone()));
                    }

                    shell.capture_event();
                }
            }
            Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
                state.keyboard_modifiers = *modifiers;
            }
            _ => {}
        };

        let status = {
            let is_hovered = cursor.is_over(layout.bounds());

            if self.on_select.is_none() {
                Status::Disabled
            } else if state.is_open {
                Status::Opened { is_hovered }
            } else if is_hovered {
                Status::Hovered
            } else {
                Status::Active
            }
        };

        if let Event::Window(window::Event::RedrawRequested(_now)) = event {
            self.last_status = Some(status);
        } else if self
            .last_status
            .is_some_and(|last_status| last_status != status)
        {
            shell.request_redraw();
        }
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let bounds = layout.bounds();
        let is_mouse_over = cursor.is_over(bounds);

        if is_mouse_over {
            if self.on_select.is_some() {
                mouse::Interaction::Pointer
            } else {
                mouse::Interaction::Idle
            }
        } else {
            mouse::Interaction::default()
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let font = self.font.unwrap_or_else(|| renderer.font());
        let selected = self.selected.as_ref().map(Borrow::borrow);
        let state = tree.state.downcast_ref::<State<Renderer::Paragraph>>();

        let bounds = layout.bounds();

        let mut style = Catalog::style(
            theme,
            &self.class,
            if state.focus.focused {
                Status::Opened { is_hovered: false }
            } else {
                self.last_status.unwrap_or(Status::Active)
            },
        );

        if state.focus.focused {
            style.border.width = 2.0;
        }
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: style.border,
                ..renderer::Quad::default()
            },
            style.background,
        );

        let handle = match &self.handle {
            Handle::Arrow { size } => Some((
                Renderer::ICON_FONT,
                Renderer::ARROW_DOWN_ICON,
                *size,
                None,
                text::Shaping::Basic,
            )),
            Handle::Static(Icon {
                font,
                code_point,
                size,
                line_height,
                shaping,
            }) => Some((*font, *code_point, *size, *line_height, *shaping)),
            Handle::Dynamic { open, closed } => {
                if state.is_open {
                    Some((
                        open.font,
                        open.code_point,
                        open.size,
                        open.line_height,
                        open.shaping,
                    ))
                } else {
                    Some((
                        closed.font,
                        closed.code_point,
                        closed.size,
                        closed.line_height,
                        closed.shaping,
                    ))
                }
            }
            Handle::None => None,
        };

        if let Some((font, code_point, size, line_height, shaping)) = handle {
            let size = size.unwrap_or_else(|| renderer.text_size());
            let line_height =
                line_height.unwrap_or_else(|| renderer.line_height());

            renderer.fill_text(
                Text {
                    content: code_point.to_string(),
                    size,
                    line_height,
                    font,
                    bounds: Size::new(
                        bounds.width,
                        f32::from(line_height.to_absolute(size)),
                    ),
                    align_x: text::Alignment::Right,
                    align_y: alignment::Vertical::Center,
                    shaping,
                    wrapping: text::Wrapping::None,
                    ellipsis: text::Ellipsis::None,
                    hint_factor: None,
                },
                Point::new(
                    bounds.x + bounds.width - self.padding.right,
                    bounds.center_y(),
                ),
                style.handle_color,
                *viewport,
            );
        }

        let label = selected.map(&self.to_string);

        if let Some(label) = label.or_else(|| self.placeholder.clone()) {
            let text_size =
                self.text_size.unwrap_or_else(|| renderer.text_size());
            let line_height =
                self.line_height.unwrap_or_else(|| renderer.line_height());

            renderer.fill_text(
                Text {
                    content: label,
                    size: text_size,
                    line_height,
                    font,
                    bounds: Size::new(
                        bounds.width - self.padding.x(),
                        f32::from(line_height.to_absolute(text_size)),
                    ),
                    align_x: text::Alignment::Default,
                    align_y: alignment::Vertical::Center,
                    shaping: self.shaping,
                    wrapping: text::Wrapping::None,
                    ellipsis: self.ellipsis,
                    hint_factor: renderer.hint_factor(),
                },
                Point::new(bounds.x + self.padding.left, bounds.center_y()),
                if selected.is_some() {
                    style.text_color
                } else {
                    style.placeholder_color
                },
                *viewport,
            );
        }
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout,
        _viewport: &Rectangle,
        _renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        let state = tree.state.downcast_mut::<State<Renderer::Paragraph>>();
        if let Some(id) = &self.id {
            state.focus.id = id.clone();
        }
        if self.on_select.is_some() {
            operation.focusable(
                Some(&state.focus.id.clone()),
                layout.bounds(),
                &mut state.focus,
            );
        }
        if !state.focus.focused {
            state.is_open = false;
        }
        let selected = self.selected.as_ref().map(Borrow::borrow);
        let label = selected.map(&self.to_string);

        if let Some(label) = label.or_else(|| self.placeholder.clone()) {
            operation.text(None, layout.bounds(), &label);
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout,
        renderer: &Renderer,
        _viewport: &Rectangle,
        translation: Vector,
        window: Size,
    ) -> Vec<overlay::Element<'b, Message, Theme, Renderer>> {
        let Some(on_select) = &self.on_select else {
            return Vec::new();
        };

        let state = tree.state.downcast_mut::<State<Renderer::Paragraph>>();
        let font = self.font.unwrap_or_else(|| renderer.font());

        if state.is_open {
            let bounds = layout.bounds();
            let position = layout.position() + translation;

            let reveal = state.reveal.then_some(state.hovered_option).flatten();
            state.reveal = false;
            let option_height = self
                .line_height
                .unwrap_or_else(|| renderer.line_height())
                .to_absolute(
                    self.text_size.unwrap_or_else(|| renderer.text_size()),
                )
                .0
                + self.padding.y();
            let mut menu = Menu::new(
                &mut state.menu,
                self.options.borrow(),
                &mut state.hovered_option,
                &self.to_string,
                |option| {
                    state.is_open = false;

                    (on_select)(option)
                },
                None,
                &self.menu_class,
            )
            .width(bounds.width)
            .height(self.menu_height)
            .padding(self.padding)
            .font(font)
            .ellipsis(self.ellipsis)
            .shaping(self.shaping);

            if let Some(text_size) = self.text_size {
                menu = menu.text_size(text_size);
            }

            let mut overlay =
                menu.overlay(renderer, position, window, bounds.height);
            if let Some(index) = reveal {
                overlay.as_overlay_mut().operate(
                    renderer,
                    &mut RevealOption {
                        index,
                        height: option_height,
                    },
                );
            }
            vec![overlay]
        } else {
            Vec::new()
        }
    }
}

impl<'a, T, L, V, Message, Theme, Renderer>
    From<PickList<'a, T, L, V, Message, Theme>>
    for Element<'a, Message, Theme, Renderer>
where
    T: Clone + PartialEq + 'a,
    L: Borrow<[T]> + 'a,
    V: Borrow<T> + 'a,
    Message: Clone + 'a,
    Theme: Catalog + 'a,
    Renderer: text::Renderer + 'a,
{
    fn from(pick_list: PickList<'a, T, L, V, Message, Theme>) -> Self {
        Self::new(pick_list)
    }
}

#[derive(Debug)]
struct State<P: text::Paragraph> {
    focus: super::Focus,
    reveal: bool,
    menu: menu::State,
    keyboard_modifiers: keyboard::Modifiers,
    is_open: bool,
    hovered_option: Option<usize>,
    options: Vec<paragraph::Plain<P>>,
    placeholder: paragraph::Plain<P>,
}

impl<P: text::Paragraph> State<P> {
    fn new() -> Self {
        Self {
            focus: super::Focus::default(),
            reveal: false,
            menu: menu::State::default(),
            keyboard_modifiers: keyboard::Modifiers::default(),
            is_open: bool::default(),
            hovered_option: Option::default(),
            options: Vec::new(),
            placeholder: paragraph::Plain::default(),
        }
    }
}

impl<P: text::Paragraph> Default for State<P> {
    fn default() -> Self {
        Self::new()
    }
}

fn move_option(
    total: usize,
    current: Option<usize>,
    key: keyboard::key::Named,
) -> Option<usize> {
    use keyboard::key::Named;
    if total == 0 {
        return None;
    }
    Some(match key {
        Named::Home => 0,
        Named::End => total - 1,
        Named::ArrowUp => current.unwrap_or(total).saturating_sub(1),
        _ => current.map_or(0, |i| (i + 1).min(total - 1)),
    })
}
struct RevealOption {
    index: usize,
    height: f32,
}
impl widget::Operation for RevealOption {
    fn traverse(&mut self, visit: &mut dyn FnMut(&mut dyn widget::Operation)) {
        visit(self);
    }
    fn scrollable(
        &mut self,
        _id: Option<&widget::Id>,
        bounds: Rectangle,
        content: Size,
        translation: Vector,
        state: &mut dyn widget::operation::Scrollable,
    ) {
        let top = self.index as f32 * self.height;
        let y = if top < translation.y {
            top
        } else if top + self.height > translation.y + bounds.height {
            top + self.height - bounds.height
        } else {
            translation.y
        };
        state.scroll_to(
            widget::operation::scrollable::AbsoluteOffset {
                x: None,
                y: Some(y),
            },
            widget::operation::Animation::Instant,
            bounds,
            content,
        );
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn key_event(key: keyboard::key::Named) -> Event {
        Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(key),
            modified_key: keyboard::Key::Named(key),
            physical_key: keyboard::key::Physical::Unidentified(
                keyboard::key::NativeCode::Unidentified,
            ),
            location: keyboard::Location::Standard,
            modifiers: keyboard::Modifiers::default(),
            text: None,
            repeat: false,
        })
    }

    #[test]
    fn popup_escape_cancels_arrows_enter_commits_and_tab_only_dismisses() {
        use iced::advanced::shell::{Bus, Waker};
        use keyboard::key::Named;
        let picker = PickList::<_, _, _, usize, iced::Theme>::new(
            Some(0),
            vec![0, 1, 2],
            usize::to_string,
        )
        .on_select(|value| value);
        let mut element: iced::Element<'_, usize, iced::Theme, ()> =
            picker.into();
        let mut tree = Tree::new(element.as_widget());
        tree.state.downcast_mut::<State<()>>().focus.focused = true;
        let viewport = Rectangle::with_size(Size::new(300.0, 400.0));
        let mut bus = Bus::new();
        let mut press = |key| {
            let mut shell = Shell::new(
                &iced::window::Headless,
                Waker::new(|| {}),
                &mut bus,
            );
            element.as_widget_mut().update(
                &mut tree,
                &key_event(key),
                Layout::new(Size::new(200.0, 30.0)),
                mouse::Cursor::Unavailable,
                &(),
                &mut shell,
                &viewport,
            );
            (
                shell.is_event_captured(),
                tree.state.downcast_ref::<State<()>>().is_open,
            )
        };
        assert_eq!(press(Named::Space), (true, true));
        assert_eq!(press(Named::ArrowDown), (true, true));
        assert_eq!(press(Named::Escape), (true, false));
        assert_eq!(press(Named::Enter), (true, true));
        assert_eq!(press(Named::ArrowDown), (true, true));
        assert_eq!(press(Named::Enter), (true, false));
        assert_eq!(press(Named::Space), (true, true));
        assert_eq!(press(Named::Tab), (false, false));
        assert_eq!(
            bus.drain().map(|(value, _)| value).collect::<Vec<_>>(),
            vec![1]
        );
    }

    #[test]
    fn clicking_an_earlier_sibling_blurs_picker_even_when_click_is_captured() {
        use iced::advanced::shell::{Bus, Waker};
        let picker = PickList::<_, _, _, usize, iced::Theme>::new(
            Some(0),
            vec![0, 1],
            usize::to_string,
        )
        .on_select(|value| value);
        let mut element: iced::Element<'_, usize, iced::Theme, ()> =
            picker.into();
        let mut tree = Tree::new(element.as_widget());
        let state = tree.state.downcast_mut::<State<()>>();
        state.focus.focused = true;
        state.is_open = true;
        let viewport = Rectangle::with_size(Size::new(300.0, 400.0));
        let mut bus = Bus::new();
        let mut shell =
            Shell::new(&iced::window::Headless, Waker::new(|| {}), &mut bus);
        shell.capture_event();
        element.as_widget_mut().update(
            &mut tree,
            &Event::Mouse(mouse::Event::ButtonPressed {
                button: mouse::Button::Left,
                modifiers: keyboard::Modifiers::default(),
            }),
            Layout::new(Size::new(200.0, 30.0)),
            mouse::Cursor::Available(Point::new(250.0, 80.0)),
            &(),
            &mut shell,
            &viewport,
        );
        let state = tree.state.downcast_ref::<State<()>>();
        assert!(!state.focus.focused);
        assert!(!state.is_open);
        assert!(bus.drain().next().is_none());
    }

    #[test]
    fn picker_keys_stop_at_edges_and_handle_empty_options() {
        use keyboard::key::Named::*;
        assert_eq!(move_option(3, Some(2), ArrowDown), Some(2));
        assert_eq!(move_option(3, Some(0), ArrowUp), Some(0));
        assert_eq!(move_option(3, Some(2), Home), Some(0));
        assert_eq!(move_option(3, Some(0), End), Some(2));
        assert_eq!(move_option(0, None, ArrowDown), None);
    }
}
