//! Installed font selection; searches never edit the configuration draft.
use iced::advanced::widget::{Operation, operation};
use iced::font::Family;
use iced::widget::{combo_box, row};
use iced::{Event, Length, Task, keyboard};

use super::{Message, labelled, style};
use crate::widget::{Element, decorate, text, text_input};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Main,
    Code,
}

impl Target {
    pub fn default_id(self) -> String {
        format!("settings-font-default-{self:?}")
    }
}

#[derive(Default)]
pub struct State {
    options: combo_box::State<String>,
    pub active: Option<Target>,
    error: Option<String>,
    loaded: bool,
}

pub fn load() -> Task<Message> {
    iced::font::list().map(|result| {
        Message::FontsLoaded(result.map_err(|error| format!("{error:?}")))
    })
}

fn names(families: Vec<Family>) -> Vec<String> {
    let mut names = families
        .into_iter()
        .filter_map(|family| match family {
            Family::Name(name)
                if !name.trim().is_empty() && !name.starts_with('.') =>
            {
                Some(name.to_owned())
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    names.sort_by_cached_key(|name| name.to_lowercase());
    names.dedup_by(|a, b| a.eq_ignore_ascii_case(b));
    names
}

impl State {
    pub fn dismiss(&mut self) {
        self.active = None;
        // Refresh the widget version so cancelled search text is discarded too.
        self.options = combo_box::State::new(self.options.options().to_vec());
    }

    pub fn loaded(&mut self, result: Result<Vec<Family>, String>) {
        self.loaded = true;
        match result {
            Ok(families) => {
                self.options = combo_box::State::new(names(families));
                self.error = self.options.options().is_empty().then(|| "No installed fonts were found. Enter a font name manually.".into());
            }
            Err(_) => self.error = Some(
                "Could not list installed fonts. Enter a font name manually."
                    .into(),
            ),
        }
    }

    pub fn view<'a>(
        &'a self,
        target: Target,
        label: &'a str,
        value: &'a String,
        id: String,
    ) -> Element<'a, Message> {
        let placeholder = if self.active == Some(target) {
            "Search installed fonts…"
        } else {
            match target {
                Target::Main => "Default (Iosevka Term)",
                Target::Code => "Use main font",
            }
        };
        let control: Element<_> =
            if self.error.is_some() {
                text_input(placeholder, value)
                    .id(id)
                    .on_input(move |value| Message::FontEdited(target, value))
                    .padding(6)
                    .style(style::input)
                    .width(Length::Fill)
                    .into()
            } else if !self.loaded {
                text_input("Loading installed fonts…", value)
                    .id(id)
                    .padding(6)
                    .style(style::input)
                    .width(Length::Fill)
                    .into()
            } else {
                let picker: Element<_> = combo_box(
                    &self.options,
                    placeholder,
                    (!value.is_empty()).then_some(value),
                    move |value| Message::FontChosen(target, value),
                )
                .id(id)
                .on_open(Message::FontOpened(target))
                .on_close(Message::FontClosed(target))
                .line_height(crate::font::current_line_height())
                .padding(6)
                .menu_height(Length::Fixed(240.0))
                .input_style(style::input)
                .menu_style(style::menu)
                .into();
                // Query actual widget focus: Tab can focus a combo box without
                // Iced emitting on_open. Keep the wrapper present in every render.
                decorate(picker)
                .update(
                    move |(): &mut (),
                          inner: &mut Element<'a, Message>,
                          tree: &mut iced::advanced::widget::Tree,
                          event: &Event,
                          layout: iced::advanced::Layout,
                          cursor: iced::mouse::Cursor,
                          renderer: &crate::widget::Renderer,
                          shell: &mut iced::advanced::Shell<'_, Message>,
                          viewport: &iced::Rectangle| {
                        let mut focused = Focused(false);
                        inner.as_widget_mut().operate(
                            tree, layout, viewport, renderer, &mut focused,
                        );
                        if focused.0
                            && let Event::Keyboard(
                                keyboard::Event::KeyPressed {
                                    key,
                                    modifiers,
                                    ..
                                },
                            ) = event
                        {
                            let message = match (key.as_ref(), *modifiers) {
                                (
                                    keyboard::Key::Named(
                                        keyboard::key::Named::Escape,
                                    ),
                                    m,
                                ) if m.is_empty() => {
                                    Some(Message::FontDismiss(target, None))
                                }
                                (
                                    keyboard::Key::Named(
                                        keyboard::key::Named::Tab,
                                    ),
                                    m,
                                ) if m.is_empty()
                                    || m == keyboard::Modifiers::SHIFT =>
                                {
                                    Some(Message::FontDismiss(
                                        target,
                                        Some(m.shift()),
                                    ))
                                }
                                _ => None,
                            };
                            if let Some(message) = message {
                                shell.publish(message);
                                shell.capture_event();
                                return;
                            }
                        }
                        inner.as_widget_mut().update(
                            tree, event, layout, cursor, renderer, shell,
                            viewport,
                        );
                    },
                )
                .into()
            };
        let reset = super::keyboard::button("Default")
            .id(target.default_id())
            .padding([6, 10])
            .style(style::button)
            .on_press(Message::FontChosen(target, String::new()));
        let mut content = iced::widget::column![labelled(
            label,
            row![control, reset].spacing(8).into()
        )]
        .spacing(4);
        if let Some(error) = &self.error {
            content = content.push(
                row![
                    text(error).size(12).style(crate::theme::text::secondary),
                    super::action("Try again", Message::FontsRetry)
                ]
                .spacing(8),
            );
        }
        content.into()
    }
}

struct Focused(bool);
impl Operation for Focused {
    fn traverse(&mut self, visit: &mut dyn FnMut(&mut dyn Operation)) {
        visit(self);
    }
    fn focusable(
        &mut self,
        _id: Option<&iced::advanced::widget::Id>,
        _bounds: iced::Rectangle,
        state: &mut dyn operation::Focusable,
    ) {
        self.0 |= state.is_focused();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discovery_lists_sorted_public_families_without_duplicate_faces() {
        assert_eq!(
            names(vec![
                Family::Name("Menlo"),
                Family::Name("Arial"),
                Family::Name("menlo"),
                Family::Name(".Hidden"),
                Family::Serif,
                Family::Name("")
            ]),
            vec!["Arial", "Menlo"]
        );
    }

    #[test]
    fn empty_or_failed_discovery_keeps_manual_entry_available() {
        let mut state = State::default();
        state.loaded(Ok(vec![]));
        assert!(state.loaded && state.error.is_some());
        state.loaded(Err("Unavailable".into()));
        assert!(state.error.is_some());
        state.loaded(Ok(vec![Family::Name("Menlo")]));
        assert!(state.error.is_none());
    }
}
