use iced::advanced::{Layout, Shell, widget};
pub use iced::keyboard::key::{self, Named, Physical};
pub use iced::keyboard::{Key, Modifiers};
use iced::{Event, Rectangle, keyboard, mouse};

use super::{Element, Renderer, decorate};

pub fn key_press<'a, Message, Theme, Renderer>(
    base: impl Into<iced::Element<'a, Message, Theme, Renderer>>,
    key: Key,
    modifiers: Modifiers,
    on_press: Message,
) -> iced::Element<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Theme: 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    key_press_with(base, key, modifiers, move || on_press.clone())
}

/// Intercept a key before child widgets using a message factory. This also
/// supports application messages that are intentionally not cloneable.
pub fn key_press_with<'a, Message, Theme, Renderer>(
    base: impl Into<iced::Element<'a, Message, Theme, Renderer>>,
    key: Key,
    modifiers: Modifiers,
    on_press: impl Fn() -> Message + 'a,
) -> iced::Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    decorate(base)
        .update(
            move |_state: &mut (),
                  inner: &mut iced::Element<'a, Message, Theme, Renderer>,
                  tree: &mut widget::Tree,
                  event: &Event,
                  layout: Layout,
                  cursor: mouse::Cursor,
                  renderer: &Renderer,
                  shell: &mut Shell<'_, Message>,
                  viewport: &Rectangle| {
                if let Event::Keyboard(keyboard::Event::KeyPressed {
                    key: k,
                    modifiers: m,
                    ..
                }) = &event
                    && key == *k
                    && modifiers == *m
                {
                    shell.publish(on_press());
                    shell.capture_event();
                    return;
                }

                inner.as_widget_mut().update(
                    tree, event, layout, cursor, renderer, shell, viewport,
                );
            },
        )
        .into()
}

/// Like [`key_press`], but the caller decides per key press which message to
/// emit (or `None` to ignore). Handy when a set of keys should be handled
/// regardless of the active modifiers.
pub fn on_key<'a, Message>(
    base: impl Into<Element<'a, Message>>,
    handler: impl Fn(&Key, Modifiers) -> Option<Message> + 'a,
) -> Element<'a, Message>
where
    Message: 'a + Clone,
{
    decorate(base)
        .update(
            move |_state: &mut (),
                  inner: &mut Element<'a, Message>,
                  tree: &mut widget::Tree,
                  event: &Event,
                  layout: Layout,
                  cursor: mouse::Cursor,
                  renderer: &Renderer,
                  shell: &mut Shell<'_, Message>,
                  viewport: &Rectangle| {
                if let Event::Keyboard(keyboard::Event::KeyPressed {
                    key,
                    modifiers,
                    ..
                }) = &event
                    && let Some(message) = handler(key, *modifiers)
                {
                    shell.publish(message);
                    shell.capture_event();
                    return;
                }

                inner.as_widget_mut().update(
                    tree, event, layout, cursor, renderer, shell, viewport,
                );
            },
        )
        .into()
}

pub fn is_numpad(physical_key: &Physical) -> bool {
    matches!(
        physical_key,
        Physical::Code(key::Code::Numpad0)
            | Physical::Code(key::Code::Numpad1)
            | Physical::Code(key::Code::Numpad2)
            | Physical::Code(key::Code::Numpad3)
            | Physical::Code(key::Code::Numpad4)
            | Physical::Code(key::Code::Numpad5)
            | Physical::Code(key::Code::Numpad6)
            | Physical::Code(key::Code::Numpad7)
            | Physical::Code(key::Code::Numpad8)
            | Physical::Code(key::Code::Numpad9)
            | Physical::Code(key::Code::NumpadAdd)
            | Physical::Code(key::Code::NumpadBackspace)
            | Physical::Code(key::Code::NumpadClear)
            | Physical::Code(key::Code::NumpadClearEntry)
            | Physical::Code(key::Code::NumpadComma)
            | Physical::Code(key::Code::NumpadDecimal)
            | Physical::Code(key::Code::NumpadDivide)
            | Physical::Code(key::Code::NumpadEnter)
            | Physical::Code(key::Code::NumpadEqual)
            | Physical::Code(key::Code::NumpadHash)
            | Physical::Code(key::Code::NumpadMemoryAdd)
            | Physical::Code(key::Code::NumpadMemoryClear)
            | Physical::Code(key::Code::NumpadMemoryRecall)
            | Physical::Code(key::Code::NumpadMemoryStore)
            | Physical::Code(key::Code::NumpadMemorySubtract)
            | Physical::Code(key::Code::NumpadMultiply)
            | Physical::Code(key::Code::NumpadParenLeft)
            | Physical::Code(key::Code::NumpadParenRight)
            | Physical::Code(key::Code::NumpadStar)
            | Physical::Code(key::Code::NumpadSubtract)
    )
}

#[cfg(test)]
mod tests {
    use iced::advanced::shell::{Bus, Waker};
    use iced::{Length, Size};

    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum Message {
        OpenSettings,
        EditorReceivedKey,
    }

    fn dispatch(modifiers: Modifiers, intercept: bool) -> Vec<Message> {
        // An editable child that would process KeyPressed.text = "," even
        // with COMMAND held. The Settings gesture must never reach it.
        let editor: iced::Element<'_, Message, iced::Theme, ()> =
            decorate(iced::widget::space().width(Length::Fill))
                .update(
                    |(): &mut (),
                     _: &mut iced::Element<'_, Message, iced::Theme, ()>,
                     _: &mut widget::Tree,
                     _: &Event,
                     _: Layout,
                     _: mouse::Cursor,
                     (): &(),
                     shell: &mut Shell<'_, Message>,
                     _: &Rectangle| {
                        shell.publish(Message::EditorReceivedKey);
                    },
                )
                .into();
        let mut element = if intercept {
            key_press(
                editor,
                Key::Character(",".into()),
                Modifiers::COMMAND,
                Message::OpenSettings,
            )
        } else {
            editor
        };
        let mut tree = widget::Tree::new(element.as_widget());
        tree.diff(element.as_widget_mut());
        let event = Event::Keyboard(keyboard::Event::KeyPressed {
            key: Key::Character(",".into()),
            modified_key: Key::Character(",".into()),
            physical_key: Physical::Code(key::Code::Comma),
            location: keyboard::Location::Standard,
            modifiers,
            text: Some(",".into()),
            repeat: false,
        });
        let mut bus = Bus::new();
        let mut shell =
            Shell::new(&iced::window::Headless, Waker::new(|| {}), &mut bus);
        element.as_widget_mut().update(
            &mut tree,
            &event,
            Layout::new(Size::new(200.0, 100.0)),
            mouse::Cursor::Unavailable,
            &(),
            &mut shell,
            &Rectangle::with_size(Size::new(200.0, 100.0)),
        );
        if intercept && modifiers == Modifiers::COMMAND {
            assert!(shell.is_event_captured());
        }
        bus.drain().map(|(message, _)| message).collect()
    }

    #[test]
    fn settings_gesture_is_consumed_before_editable_child_and_runs_once() {
        assert_eq!(
            dispatch(Modifiers::COMMAND, true),
            vec![Message::OpenSettings]
        );
        assert_eq!(
            dispatch(Modifiers::empty(), true),
            vec![Message::EditorReceivedKey]
        );
        assert_eq!(
            dispatch(Modifiers::COMMAND | Modifiers::SHIFT, true),
            vec![Message::EditorReceivedKey]
        );
    }

    #[test]
    fn explicit_binding_can_disable_settings_interceptor() {
        assert_eq!(
            dispatch(Modifiers::COMMAND, false),
            vec![Message::EditorReceivedKey]
        );
    }
}
