//! Settings keeps the chosen palette, with readable text and visible controls.
use iced::widget::overlay::menu;
use iced::widget::{
    button as button_widget, checkbox as checkbox_widget,
    pick_list as pick_list_widget, scrollable as scrollable_widget,
};
use iced::{Border, Color, Shadow};

use crate::widget::text_input;
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

// Contrast between surfaces is deliberately gentler than text contrast, but
// still visible when a theme supplies identical window and input backgrounds.
fn surface(background: Color, foreground: Color, minimum: f32) -> Color {
    (0..=100)
        .map(|step| background.mix(foreground, step as f32 / 100.0))
        .find(|candidate| candidate.relative_contrast(background) >= minimum)
        .unwrap_or_else(|| contrast(foreground, background, minimum))
}

fn pane_background(theme: &Theme) -> Color {
    surface(
        theme.styles().general.background,
        theme.styles().text.primary.color,
        1.06,
    )
}

fn control_background(theme: &Theme, hovered: bool) -> Color {
    let pane = pane_background(theme);
    let minimum = if hovered { 1.22 } else { 1.16 };
    let supplied = opaque(theme.styles().buffer.background_text_input, pane);
    if supplied.relative_contrast(pane) >= minimum {
        supplied
    } else {
        surface(pane, theme.styles().text.primary.color, minimum)
    }
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
    let pane_background = surface(background, styles.text.primary.color, 1.06);
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
    let outside = pane_background(theme);
    let minimum = if focused { 4.5 } else { 3.0 };
    border.color = (0..=100)
        .flat_map(|step| {
            [Color::BLACK, Color::WHITE]
                .map(|target| border.color.mix(target, step as f32 / 100.0))
        })
        .find(|candidate| {
            candidate.relative_contrast(background) >= 3.0
                && candidate.relative_contrast(outside) >= minimum
                && candidate
                    .relative_contrast(theme.styles().general.background)
                    >= minimum
        })
        .unwrap_or_else(|| contrast(border.color, background, 3.0));
    border
}

pub fn input(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let background = control_background(
        theme,
        matches!(status, text_input::Status::Hovered),
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
    let background = control_background(
        theme,
        !matches!(
            status,
            pick_list_widget::Status::Active
                | pick_list_widget::Status::Disabled
        ),
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
    let background = control_background(theme, false);
    let selected = surface(background, theme.styles().text.primary.color, 1.3);
    menu::Style {
        background: background.into(),
        border: border_on(theme, background, false),
        text_color: contrast(
            theme.styles().text.primary.color,
            background,
            4.5,
        ),
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
    let background = pane_background(theme);
    iced::widget::container::Style {
        background: Some(background.into()),
        border: Border {
            color: contrast(fill(theme, 0.24), background, 1.5),
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
    let background = surface(
        pane_background(theme),
        theme.styles().text.primary.color,
        if status == button_widget::Status::Hovered {
            1.16
        } else if selected {
            1.12
        } else {
            1.0
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
            radius: 4.0.into(),
            ..Border::default()
        },
        ..Default::default()
    }
}

pub fn tab(
    theme: &Theme,
    status: button_widget::Status,
    selected: bool,
) -> button_widget::Style {
    let background = surface(
        pane_background(theme),
        theme.styles().text.primary.color,
        if selected {
            1.3
        } else if status == button_widget::Status::Hovered {
            1.16
        } else {
            1.0
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
                border_on(theme, background, false).color
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
        control_background(theme, false)
    };
    checkbox_widget::Style {
        background: background.into(),
        border: border_on(theme, background, false),
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
        for background in [
            Color::WHITE,
            Color::from_rgb(0.06, 0.12, 0.15),
            Color::from_rgb(0.5, 0.5, 0.5),
        ] {
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
            let pane = pane_background(&adjusted);
            assert!(pane.relative_contrast(background) >= 1.06);
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
            assert!(surface.relative_contrast(pane) >= 1.16);
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
            let selected = tab(&adjusted, button_widget::Status::Active, true);
            let Some(Background::Color(surface)) = selected.background else {
                panic!("solid selected group surface");
            };
            assert!(surface.relative_contrast(pane) >= 1.3);
            assert!(selected.text_color.relative_contrast(surface) >= 4.5);
            for status in [
                pick_list_widget::Status::Active,
                pick_list_widget::Status::Hovered,
                pick_list_widget::Status::Opened { is_hovered: false },
            ] {
                let picker = pick_list(&adjusted, status);
                let Background::Color(surface) = picker.background else {
                    panic!("solid picker surface");
                };
                assert!(surface.relative_contrast(pane) >= 1.16);
                assert!(picker.text_color.relative_contrast(surface) >= 4.5);
                assert!(picker.border.color.relative_contrast(surface) >= 3.0);
            }
        }
    }
}
