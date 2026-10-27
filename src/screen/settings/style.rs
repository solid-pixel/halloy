//! Settings keeps the chosen palette, with readable text and visible controls.
use iced::widget::overlay::menu;
use iced::widget::{
    button as button_widget, checkbox as checkbox_widget,
    pick_list as pick_list_widget, scrollable as scrollable_widget, text_input,
};
use iced::{Border, Color, Shadow};

use crate::{Theme, theme};

fn opaque(color: Color, background: Color) -> Color {
    Color::from_rgb(
        color.r * color.a + background.r * (1.0 - color.a),
        color.g * color.a + background.g * (1.0 - color.a),
        color.b * color.a + background.b * (1.0 - color.a),
    )
}

fn contrast(color: Color, background: Color, minimum: f32) -> Color {
    let color = opaque(color, background);
    let target = if Color::WHITE.relative_contrast(background)
        > Color::BLACK.relative_contrast(background)
    {
        Color::WHITE
    } else {
        Color::BLACK
    };
    (0..=100)
        .map(|step| color.mix(target, step as f32 / 100.0))
        .find(|candidate| candidate.relative_contrast(background) >= minimum)
        .unwrap_or(target)
}

pub fn accessible_theme(theme: &Theme) -> Theme {
    let mut theme = theme.clone();
    let data = match &mut theme {
        Theme::Selected(theme) | Theme::Preview { preview: theme, .. } => theme,
    };
    let styles = &mut data.styles;
    let background = opaque(styles.general.background, Color::BLACK);
    styles.general.background = background;
    for color in [
        &mut styles.text.primary.color,
        &mut styles.text.secondary.color,
        &mut styles.text.tertiary.color,
        &mut styles.text.success.color,
        &mut styles.text.error.color,
    ] {
        *color = contrast(*color, background, 4.5);
    }
    styles.text.primary.color =
        contrast(styles.text.primary.color, background, 7.0);
    let pane_background = background.mix(styles.text.primary.color, 0.025);
    for color in [
        &mut styles.text.secondary.color,
        &mut styles.text.tertiary.color,
        &mut styles.text.success.color,
        &mut styles.text.error.color,
    ] {
        *color = contrast(*color, pane_background, 4.5);
    }
    styles.general.border = contrast(styles.general.border, background, 3.4);
    styles.general.horizontal_rule =
        contrast(styles.general.horizontal_rule, background, 1.5);
    styles.general.scrollbar = Some(contrast(
        styles.general.scrollbar.unwrap_or(styles.general.border),
        background,
        3.0,
    ));
    theme
}

fn fill(theme: &Theme, amount: f32) -> Color {
    theme
        .styles()
        .general
        .background
        .mix(theme.styles().text.primary.color, amount)
}

fn border(theme: &Theme, focused: bool) -> Border {
    Border {
        radius: 4.0.into(),
        width: if focused { 2.0 } else { 1.0 },
        color: if focused {
            contrast(
                theme.styles().buffer.url.color,
                theme.styles().general.background,
                4.5,
            )
        } else {
            theme.styles().general.border
        },
    }
}

fn border_on(theme: &Theme, background: Color, focused: bool) -> Border {
    let mut border = border(theme, focused);
    border.color =
        contrast(border.color, background, if focused { 4.5 } else { 3.0 });
    border
}

pub fn input(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let background = fill(
        theme,
        if matches!(status, text_input::Status::Hovered) {
            0.07
        } else {
            0.04
        },
    );
    text_input::Style {
        background: background.into(),
        border: border_on(
            theme,
            background,
            matches!(status, text_input::Status::Focused { .. }),
        ),
        placeholder: contrast(
            theme.styles().text.secondary.color,
            background,
            4.5,
        ),
        value: contrast(theme.styles().text.primary.color, background, 4.5),
        selection: theme.styles().buffer.selection,
    }
}

pub fn pick_list(
    theme: &Theme,
    status: pick_list_widget::Status,
) -> pick_list_widget::Style {
    let background = fill(
        theme,
        if matches!(
            status,
            pick_list_widget::Status::Active
                | pick_list_widget::Status::Disabled
        ) {
            0.04
        } else {
            0.07
        },
    );
    pick_list_widget::Style {
        background: background.into(),
        border: border_on(
            theme,
            background,
            matches!(status, pick_list_widget::Status::Opened { .. }),
        ),
        text_color: contrast(
            theme.styles().text.primary.color,
            background,
            4.5,
        ),
        placeholder_color: contrast(
            theme.styles().text.secondary.color,
            background,
            4.5,
        ),
        handle_color: contrast(
            theme.styles().text.primary.color,
            background,
            4.5,
        ),
    }
}

pub fn menu(theme: &Theme) -> menu::Style {
    let background = theme.styles().general.background;
    let selected = fill(theme, 0.12);
    menu::Style {
        background: background.into(),
        border: border(theme, false),
        text_color: theme.styles().text.primary.color,
        selected_background: selected.into(),
        selected_text_color: contrast(
            theme.styles().text.primary.color,
            selected,
            4.5,
        ),
        shadow: Shadow::default(),
    }
}

pub fn button(
    theme: &Theme,
    status: button_widget::Status,
) -> button_widget::Style {
    let disabled = status == button_widget::Status::Disabled;
    let background = fill(
        theme,
        match status {
            button_widget::Status::Active | button_widget::Status::Disabled => {
                0.04
            }
            button_widget::Status::Hovered => 0.09,
            button_widget::Status::Pressed => 0.14,
        },
    );
    button_widget::Style {
        background: Some(background.into()),
        text_color: contrast(
            if disabled {
                theme.styles().text.secondary.color
            } else {
                theme.styles().text.primary.color
            },
            background,
            4.5,
        ),
        border: if disabled {
            Border {
                color: fill(theme, 0.18),
                ..border(theme, false)
            }
        } else {
            border(theme, false)
        },
        ..Default::default()
    }
}

pub fn primary_button(
    theme: &Theme,
    status: button_widget::Status,
) -> button_widget::Style {
    if status == button_widget::Status::Disabled {
        return button(theme, status);
    }
    let background = contrast(
        theme.styles().buffer.url.color,
        theme.styles().general.background,
        3.0,
    );
    let background = if status == button_widget::Status::Hovered {
        background.mix(theme.styles().text.primary.color, 0.1)
    } else {
        background
    };
    button_widget::Style {
        background: Some(background.into()),
        text_color: contrast(
            theme.styles().general.background,
            background,
            4.5,
        ),
        border: Border {
            color: background,
            ..border(theme, false)
        },
        ..Default::default()
    }
}

pub fn pane(theme: &Theme) -> iced::widget::container::Style {
    iced::widget::container::Style {
        background: Some(fill(theme, 0.025).into()),
        border: Border {
            color: fill(theme, 0.18),
            width: 1.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    }
}

pub fn section_tab(
    theme: &Theme,
    status: button_widget::Status,
    selected: bool,
) -> button_widget::Style {
    let background = fill(
        theme,
        if status == button_widget::Status::Hovered {
            0.07
        } else if selected {
            0.025
        } else {
            0.0
        },
    );
    button_widget::Style {
        background: Some(background.into()),
        text_color: contrast(
            theme.styles().text.primary.color,
            background,
            4.5,
        ),
        border: Border {
            radius: iced::border::Radius {
                top_left: 4.0,
                top_right: 4.0,
                bottom_left: 0.0,
                bottom_right: 0.0,
            },
            ..Border::default()
        },
        ..Default::default()
    }
}

pub fn tab_indicator(
    theme: &Theme,
    selected: bool,
) -> iced::widget::container::Style {
    iced::widget::container::Style {
        background: selected.then(|| {
            contrast(
                theme.styles().buffer.url.color,
                theme.styles().general.background,
                3.0,
            )
            .into()
        }),
        ..Default::default()
    }
}

pub fn tab(
    theme: &Theme,
    status: button_widget::Status,
    selected: bool,
) -> button_widget::Style {
    let background = fill(
        theme,
        if selected {
            0.14
        } else if status == button_widget::Status::Hovered {
            0.07
        } else {
            0.0
        },
    );
    button_widget::Style {
        background: Some(background.into()),
        text_color: contrast(
            theme.styles().text.primary.color,
            background,
            4.5,
        ),
        border: Border {
            color: if selected {
                theme.styles().general.border
            } else {
                Color::TRANSPARENT
            },
            ..border(theme, false)
        },
        ..Default::default()
    }
}

pub fn checkbox(
    theme: &Theme,
    status: checkbox_widget::Status,
) -> checkbox_widget::Style {
    let checked = match status {
        checkbox_widget::Status::Active { is_checked }
        | checkbox_widget::Status::Hovered { is_checked }
        | checkbox_widget::Status::Disabled { is_checked } => is_checked,
    };
    let background = if checked {
        contrast(
            theme.styles().buffer.url.color,
            theme.styles().general.background,
            3.0,
        )
    } else {
        fill(theme, 0.04)
    };
    checkbox_widget::Style {
        background: background.into(),
        border: border(theme, false),
        icon_color: contrast(
            theme.styles().general.background,
            background,
            4.5,
        ),
        text_color: Some(theme.styles().text.primary.color),
    }
}

pub fn scrollable(
    theme: &Theme,
    status: scrollable_widget::Status,
) -> scrollable_widget::Style {
    theme::scrollable::primary(theme, status)
}

#[cfg(test)]
mod tests {
    use iced::Background;

    use super::*;

    #[test]
    fn settings_controls_remain_readable_with_low_contrast_light_and_dark_palettes()
     {
        for background in [Color::WHITE, Color::from_rgb(0.06, 0.12, 0.15)] {
            let mut styles = data::appearance::theme::Styles::default();
            styles.general.background = background;
            styles.general.border = background;
            styles.text.primary.color = background;
            styles.text.secondary.color = Color {
                a: 0.2,
                ..background
            };
            styles.text.tertiary.color = background;
            let original =
                Theme::from(data::Theme::new("Low contrast".into(), styles));
            let adjusted = accessible_theme(&original);
            assert_eq!(original.styles().text.primary.color, background);
            for color in [
                adjusted.styles().text.primary.color,
                adjusted.styles().text.secondary.color,
                adjusted.styles().text.tertiary.color,
            ] {
                assert!(color.relative_contrast(background) >= 4.5);
            }
            let input_style = input(&adjusted, text_input::Status::Active);
            let Background::Color(surface) = input_style.background else {
                panic!("solid input surface");
            };
            assert!(input_style.value.relative_contrast(surface) >= 4.5);
            assert!(input_style.placeholder.relative_contrast(surface) >= 4.5);
            assert!(input_style.border.color.relative_contrast(surface) >= 3.0);
            assert!(
                input_style.border.color.relative_contrast(background) >= 3.0
            );
            let disabled = button(&adjusted, button_widget::Status::Disabled);
            let Some(Background::Color(surface)) = disabled.background else {
                panic!("solid button surface");
            };
            assert!(disabled.text_color.relative_contrast(surface) >= 4.5);
            assert_eq!(input_style.border.width, 1.0);
            assert_eq!(
                input(
                    &adjusted,
                    text_input::Status::Focused { is_hovered: false }
                )
                .border
                .width,
                2.0
            );
        }
    }
}
