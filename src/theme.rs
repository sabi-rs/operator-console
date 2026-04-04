use ratatui::style::Color;
use std::str::FromStr;
use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Name {
    Dragon = 0,
    Wave = 1,
}

impl Name {
    pub fn all() -> &'static [Name] {
        &[Name::Dragon, Name::Wave]
    }

    pub fn slug(&self) -> &'static str {
        match self {
            Self::Dragon => "kanagawa-dragon",
            Self::Wave => "kanagawa-wave",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Dragon => "Kanagawa Dragon",
            Self::Wave => "Kanagawa Wave",
        }
    }
}

impl FromStr for Name {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "dragon" | "kanagawa-dragon" => Ok(Name::Dragon),
            "wave" | "kanagawa-wave" | "terminal" => Ok(Name::Wave),
            _ => Err(format!(
                "unknown theme `{value}`; run --list-themes to see valid theme names"
            )),
        }
    }
}

#[derive(Clone, Copy)]
struct Palette {
    shell_background: Color,
    panel_background: Color,
    elevated_background: Color,
    text_color: Color,
    muted_text: Color,
    border_color: Color,
    accent_blue: Color,
    accent_cyan: Color,
    accent_green: Color,
    accent_gold: Color,
    accent_pink: Color,
    accent_red: Color,
    selected_background: Color,
    selected_text: Color,
    contrast_text: Color,
}

const DRAGON: Palette = Palette {
    shell_background: Color::Rgb(18, 20, 20),
    panel_background: Color::Rgb(24, 22, 22),
    elevated_background: Color::Rgb(34, 30, 34),
    text_color: Color::Rgb(197, 201, 197),
    muted_text: Color::Rgb(114, 113, 105),
    border_color: Color::Rgb(84, 84, 100),
    accent_blue: Color::Rgb(139, 164, 176),
    accent_cyan: Color::Rgb(142, 164, 162),
    accent_green: Color::Rgb(138, 154, 123),
    accent_gold: Color::Rgb(196, 178, 138),
    accent_pink: Color::Rgb(162, 146, 163),
    accent_red: Color::Rgb(196, 116, 110),
    selected_background: Color::Rgb(45, 79, 103),
    selected_text: Color::Rgb(200, 204, 200),
    contrast_text: Color::Rgb(18, 20, 20),
};

const WAVE: Palette = Palette {
    shell_background: Color::Rgb(29, 32, 39),
    panel_background: Color::Rgb(34, 36, 49),
    elevated_background: Color::Rgb(41, 44, 60),
    text_color: Color::Rgb(220, 215, 186),
    muted_text: Color::Rgb(127, 132, 156),
    border_color: Color::Rgb(84, 88, 112),
    accent_blue: Color::Rgb(124, 153, 199),
    accent_cyan: Color::Rgb(122, 168, 196),
    accent_green: Color::Rgb(152, 187, 108),
    accent_gold: Color::Rgb(229, 192, 123),
    accent_pink: Color::Rgb(211, 188, 192),
    accent_red: Color::Rgb(228, 104, 118),
    selected_background: Color::Rgb(58, 74, 104),
    selected_text: Color::Rgb(236, 229, 199),
    contrast_text: Color::Rgb(29, 32, 39),
};

static CURRENT_THEME: AtomicU8 = AtomicU8::new(Name::Wave as u8);

pub fn default_theme() -> Name {
    Name::Wave
}

pub fn set_theme(name: Name) {
    CURRENT_THEME.store(name as u8, Ordering::Relaxed);
}

pub fn theme_name() -> Name {
    match CURRENT_THEME.load(Ordering::Relaxed) {
        0 => Name::Dragon,
        1 => Name::Wave,
        _ => default_theme(),
    }
}

fn palette_for(name: Name) -> &'static Palette {
    match name {
        Name::Dragon => &DRAGON,
        Name::Wave => &WAVE,
    }
}

fn current_palette() -> &'static Palette {
    palette_for(theme_name())
}

pub fn shell_background() -> Color {
    current_palette().shell_background
}

pub fn panel_background() -> Color {
    current_palette().panel_background
}

pub fn elevated_background() -> Color {
    current_palette().elevated_background
}

pub fn text_color() -> Color {
    current_palette().text_color
}

pub fn muted_text() -> Color {
    current_palette().muted_text
}

pub fn border_color() -> Color {
    current_palette().border_color
}

pub fn accent_blue() -> Color {
    current_palette().accent_blue
}

pub fn accent_cyan() -> Color {
    current_palette().accent_cyan
}

pub fn accent_green() -> Color {
    current_palette().accent_green
}

pub fn accent_gold() -> Color {
    current_palette().accent_gold
}

pub fn accent_pink() -> Color {
    current_palette().accent_pink
}

pub fn accent_red() -> Color {
    current_palette().accent_red
}

pub fn selected_background() -> Color {
    current_palette().selected_background
}

pub fn selected_text() -> Color {
    current_palette().selected_text
}

pub fn contrast_text(_background: Color) -> Color {
    current_palette().contrast_text
}
