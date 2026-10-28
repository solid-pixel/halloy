use data::shortcut::KeyBind;
use iced::{Subscription, event, keyboard, mouse, window};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    OpenSettings,
    Copy,
    Escape,
    LeftClick,
    ResetFocus,
    UpdatePrimaryClipboard,
    /// An ignored key press for the dashboard to match on
    Key(KeyBind),
}

impl Event {
    /// Focus actions are handled after widget dispatch, so they cannot mark
    /// their key press captured. Honor those explicit bindings too.
    pub fn resolve_settings_shortcut(
        self,
        keyboard: &data::config::keys::Keyboard,
    ) -> Self {
        if self == Self::OpenSettings {
            let binding = KeyBind::from((
                iced::keyboard::Key::Character(",".into()),
                iced::keyboard::Modifiers::COMMAND,
            ));
            if keyboard.focus_command(&binding, false).is_some() {
                return Self::Key(binding);
            }
        }
        self
    }
}

pub fn events() -> Subscription<(window::Id, Event)> {
    event::listen_with(filtered_events)
}

fn filtered_events(
    event: iced::Event,
    status: iced::event::Status,
    window: window::Id,
) -> Option<(window::Id, Event)> {
    let ignored = |status| matches!(status, iced::event::Status::Ignored);

    let recognized = match &event {
        iced::Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Character(key),
            modifiers,
            ..
        }) if key.as_str() == ","
            && *modifiers == keyboard::Modifiers::COMMAND
            && ignored(status) =>
        {
            Some(Event::OpenSettings)
        }
        iced::Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(keyboard::key::Named::Escape),
            ..
        }) => Some(Event::Escape),
        iced::Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Character(c),
            modifiers,
            ..
        }) if c.as_str() == "c" && modifiers.command() => Some(Event::Copy),
        iced::Event::Mouse(mouse::Event::ButtonPressed {
            button: mouse::Button::Left,
            ..
        }) if ignored(status) => Some(Event::LeftClick),
        iced::Event::Mouse(mouse::Event::ButtonReleased(
            mouse::Button::Left,
        )) if cfg!(target_os = "linux") && ignored(status) => {
            Some(Event::UpdatePrimaryClipboard)
        }
        // Any other mouse press (a captured left click, or a right click)
        // exits message focus
        iced::Event::Mouse(mouse::Event::ButtonPressed {
            button: mouse::Button::Left | mouse::Button::Right,
            ..
        }) => Some(Event::ResetFocus),
        // Forward ignored key presses so the dashboard can match them
        iced::Event::Keyboard(keyboard::Event::KeyPressed {
            key,
            modifiers,
            ..
        }) if ignored(status) => {
            Some(Event::Key(KeyBind::from((key.clone(), *modifiers))))
        }
        _ => None,
    };

    recognized.map(|event| (window, event))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn comma(modifiers: keyboard::Modifiers) -> iced::Event {
        iced::Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Character(",".into()),
            modified_key: keyboard::Key::Character(",".into()),
            physical_key: keyboard::key::Physical::Code(
                keyboard::key::Code::Comma,
            ),
            location: keyboard::Location::Standard,
            modifiers,
            text: None,
            repeat: false,
        })
    }

    #[test]
    fn settings_shortcut_only_handles_an_uncaptured_exact_binding() {
        let window = window::Id::unique();
        let command = keyboard::Modifiers::COMMAND;
        assert_eq!(
            filtered_events(comma(command), event::Status::Ignored, window),
            Some((window, Event::OpenSettings))
        );
        // An explicit user binding has already run through the dashboard.
        assert_eq!(
            filtered_events(comma(command), event::Status::Captured, window),
            None
        );
        assert!(!matches!(
            filtered_events(
                comma(command | keyboard::Modifiers::SHIFT),
                event::Status::Ignored,
                window
            ),
            Some((_, Event::OpenSettings))
        ));
    }

    #[test]
    fn settings_shortcut_defers_to_configured_focus_actions() {
        let mut config = data::config::keys::Keyboard::default();
        assert_eq!(
            Event::OpenSettings.resolve_settings_shortcut(&config),
            Event::OpenSettings
        );
        let binding = KeyBind::from((
            keyboard::Key::Character(",".into()),
            keyboard::Modifiers::COMMAND,
        ));
        config.focus_reply = binding.clone().into();
        assert_eq!(
            Event::OpenSettings.resolve_settings_shortcut(&config),
            Event::Key(binding)
        );
    }
}
