use std::borrow::Cow;
use std::cell::RefCell;
use std::sync::{LazyLock, RwLock};

use data::appearance::theme::FontStyle;
use data::config;
use hashbrown::HashMap;
use iced::font;
use iced::widget::text::LineHeight;

pub static PRIMARY: Font = Font::new(false, false);
pub static PRIMARY_BOLD: Font = Font::new(true, false);
pub static PRIMARY_ITALICS: Font = Font::new(false, true);
pub static PRIMARY_BOLD_ITALICS: Font = Font::new(true, true);
pub static CODE: Font = Font::new(false, false);
pub static CODE_BOLD: Font = Font::new(true, false);
pub static CODE_ITALICS: Font = Font::new(false, true);
pub static CODE_BOLD_ITALICS: Font = Font::new(true, true);
pub static ICON: LazyLock<iced::Font> =
    LazyLock::new(|| iced::Font::with_family("halloy-icons"));
pub const MESSAGE_MARKER_FONT_SCALE: f32 = 1.33;

const WIDTH_CACHE_CAPACITY: usize = 4096;

thread_local! {
    static WIDTHS: RefCell<HashMap<u32, HashMap<String, f32>>> =
        RefCell::new(HashMap::new());
    static MESSAGE_MARKER_WIDTHS: RefCell<HashMap<u32, f32>> =
        RefCell::new(HashMap::new());
}

#[derive(Debug)]
pub struct Font {
    bold: bool,
    italics: bool,
    inner: RwLock<Option<iced::Font>>,
}

impl Clone for Font {
    fn clone(&self) -> Self {
        Self {
            bold: self.bold,
            italics: self.italics,
            inner: RwLock::new(*self.inner.read().expect("font lock")),
        }
    }
}

static LINE_HEIGHT: RwLock<LineHeight> = RwLock::new(LineHeight::Relative(1.3));

pub fn current_line_height() -> LineHeight {
    *LINE_HEIGHT.read().expect("line height lock")
}

impl Font {
    const fn new(bold: bool, italics: bool) -> Self {
        Self {
            bold,
            italics,
            inner: RwLock::new(None),
        }
    }

    fn set(
        &self,
        font: iced::Font,
        stretch: font::Stretch,
        weight: font::Weight,
        bold_weight: font::Weight,
    ) {
        let weight = if self.bold { bold_weight } else { weight };
        let style = if self.italics {
            font::Style::Italic
        } else {
            font::Style::Normal
        };

        *self.inner.write().expect("font lock") = Some(iced::Font {
            stretch,
            weight,
            style,
            ..font
        });
    }
}

impl From<Font> for iced::Font {
    fn from(value: Font) -> Self {
        value
            .inner
            .into_inner()
            .expect("font lock")
            .expect("font is set on startup")
    }
}

pub fn line_height(config: &config::Font) -> LineHeight {
    config
        .line_height
        .map(LineHeight::Relative)
        .unwrap_or_default()
}

pub fn shaping() -> iced::advanced::text::Shaping {
    #[cfg(debug_assertions)]
    {
        iced::advanced::text::Shaping::Basic
    }

    #[cfg(not(debug_assertions))]
    {
        iced::advanced::text::Shaping::Advanced
    }
}

fn default_font() -> iced::Font {
    #[cfg(feature = "iosevka-font")]
    {
        iced::Font::with_family("Iosevka Term")
    }

    #[cfg(not(feature = "iosevka-font"))]
    {
        iced::Font::MONOSPACE
    }
}

pub fn set(config: &config::Font) {
    *LINE_HEIGHT.write().expect("line height lock") = line_height(config);
    WIDTHS.with_borrow_mut(HashMap::clear);
    let font = config.family.as_ref().map_or_else(default_font, |family| {
        iced::Font::with_family(family.as_str())
    });
    let code_font = config
        .code
        .as_ref()
        .map_or(font, |family| iced::Font::with_family(family.as_str()));

    let bold_weight = config.bold_weight.unwrap_or(match config.weight {
        font::Weight::Thin => font::Weight::Normal,
        font::Weight::ExtraLight => font::Weight::Medium,
        font::Weight::Light => font::Weight::Semibold,
        font::Weight::Normal => font::Weight::Bold,
        font::Weight::Medium => font::Weight::ExtraBold,
        font::Weight::Semibold
        | font::Weight::Bold
        | font::Weight::ExtraBold
        | font::Weight::Black => font::Weight::Black,
    });

    PRIMARY.set(font, config.stretch, config.weight, bold_weight);
    PRIMARY_BOLD.set(font, config.stretch, config.weight, bold_weight);
    PRIMARY_ITALICS.set(font, config.stretch, config.weight, bold_weight);
    PRIMARY_BOLD_ITALICS.set(font, config.stretch, config.weight, bold_weight);
    CODE.set(code_font, config.stretch, config.weight, bold_weight);
    CODE_BOLD.set(code_font, config.stretch, config.weight, bold_weight);
    CODE_ITALICS.set(code_font, config.stretch, config.weight, bold_weight);
    CODE_BOLD_ITALICS.set(
        code_font,
        config.stretch,
        config.weight,
        bold_weight,
    );
}

/// Refresh explicit faces, measured widths, and the defaults of every open
/// window. The runtime also uses these defaults for subsequently opened windows.
pub fn apply<Message>(config: &config::Font) -> iced::Task<Message> {
    set(config);
    iced::font::set_defaults(
        PRIMARY.clone().into(),
        config.size.map_or(crate::theme::TEXT_SIZE, f32::from),
    )
}

pub fn load() -> Vec<Cow<'static, [u8]>> {
    vec![
        #[cfg(feature = "iosevka-font")]
        include_bytes!("../fonts/iosevka-term-regular.ttf")
            .as_slice()
            .into(),
        #[cfg(feature = "iosevka-font")]
        include_bytes!("../fonts/iosevka-term-bold.ttf")
            .as_slice()
            .into(),
        #[cfg(feature = "iosevka-font")]
        include_bytes!("../fonts/iosevka-term-italic.ttf")
            .as_slice()
            .into(),
        #[cfg(feature = "iosevka-font")]
        include_bytes!("../fonts/iosevka-term-light.ttf")
            .as_slice()
            .into(),
        #[cfg(feature = "iosevka-font")]
        include_bytes!("../fonts/iosevka-term-semibold.ttf")
            .as_slice()
            .into(),
        #[cfg(feature = "iosevka-font")]
        include_bytes!("../fonts/iosevka-term-lightitalic.ttf")
            .as_slice()
            .into(),
        include_bytes!("../fonts/halloy-icons.ttf")
            .as_slice()
            .into(),
    ]
}

pub fn width_from_str(text: &str, config: &config::Font) -> f32 {
    let size = config.size.map_or(crate::theme::TEXT_SIZE, f32::from);

    WIDTHS.with_borrow_mut(|widths| {
        let widths = widths.entry(size.to_bits()).or_default();

        if let Some(width) = widths.get(text) {
            return *width;
        }

        if widths.len() >= WIDTH_CACHE_CAPACITY {
            widths.clear();
        }

        let width = measure_width(text, size);
        widths.insert(text.to_owned(), width);
        width
    })
}

fn measure_width(text: &str, size: f32) -> f32 {
    use iced::advanced::graphics::text::Paragraph;
    use iced::advanced::text::{self, Paragraph as _, Text};
    use iced::{Size, alignment};

    Paragraph::with_text(Text {
        content: text,
        bounds: Size::INFINITE,
        size: size.into(),
        line_height: LineHeight::default(),
        font: PRIMARY.clone().into(),
        align_x: text::Alignment::Right,
        align_y: alignment::Vertical::Top,
        shaping: text::Shaping::Basic,
        wrapping: text::Wrapping::default(),
        ellipsis: text::Ellipsis::default(),
        hint_factor: None,
    })
    .min_bounds()
    .width
}

pub fn width_of_message_marker(config: &config::Font) -> f32 {
    let font_size = config.size.map_or(crate::theme::TEXT_SIZE, f32::from)
        * MESSAGE_MARKER_FONT_SCALE;

    MESSAGE_MARKER_WIDTHS.with_borrow_mut(|widths| {
        *widths
            .entry(font_size.to_bits())
            .or_insert_with(|| measure_message_marker(font_size))
    })
}

fn measure_message_marker(font_size: f32) -> f32 {
    use iced::advanced::graphics::text::Paragraph;
    use iced::advanced::text::{self, Paragraph as _, Text};
    use iced::{Size, alignment};

    Paragraph::with_text(Text {
        content: "\u{E000}",
        bounds: Size::INFINITE,
        size: font_size.into(),
        line_height: LineHeight::default(),
        font: *ICON,
        align_x: text::Alignment::Right,
        align_y: alignment::Vertical::Top,
        shaping: text::Shaping::Basic,
        wrapping: text::Wrapping::default(),
        ellipsis: text::Ellipsis::default(),
        hint_factor: None,
    })
    .min_bounds()
    .width
}

pub fn get(font_style: FontStyle) -> Font {
    match font_style {
        FontStyle::Normal => PRIMARY.clone(),
        FontStyle::Bold => PRIMARY_BOLD.clone(),
        FontStyle::Italic => PRIMARY_ITALICS.clone(),
        FontStyle::ItalicBold => PRIMARY_BOLD_ITALICS.clone(),
    }
}

pub fn get_code(font_style: FontStyle) -> Font {
    match font_style {
        FontStyle::Normal => CODE.clone(),
        FontStyle::Bold => CODE_BOLD.clone(),
        FontStyle::Italic => CODE_ITALICS.clone(),
        FontStyle::ItalicBold => CODE_BOLD_ITALICS.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn faces_can_be_changed_repeatedly_without_losing_style_or_old_snapshots() {
        for (bold, italics) in
            [(false, false), (true, false), (false, true), (true, true)]
        {
            let face = Font::new(bold, italics);
            face.set(
                iced::Font::MONOSPACE,
                font::Stretch::Normal,
                font::Weight::Normal,
                font::Weight::Bold,
            );
            let before: iced::Font = face.clone().into();
            face.set(
                iced::Font::with_family("Georgia"),
                font::Stretch::Condensed,
                font::Weight::Light,
                font::Weight::Semibold,
            );
            let after: iced::Font = face.clone().into();
            assert_ne!(before.family, after.family);
            assert_eq!(after.stretch, font::Stretch::Condensed);
            assert_eq!(
                after.weight,
                if bold {
                    font::Weight::Semibold
                } else {
                    font::Weight::Light
                }
            );
            assert_eq!(
                after.style,
                if italics {
                    font::Style::Italic
                } else {
                    font::Style::Normal
                }
            );
            assert_eq!(
                before.weight,
                if bold {
                    font::Weight::Bold
                } else {
                    font::Weight::Normal
                }
            );
            face.set(
                iced::Font::MONOSPACE,
                font::Stretch::Normal,
                font::Weight::Normal,
                font::Weight::Bold,
            );
            assert_eq!(iced::Font::from(face), before);
        }
    }
}
