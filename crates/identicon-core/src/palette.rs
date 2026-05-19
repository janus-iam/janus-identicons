#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Palette {
    pub name: &'static str,
    pub bg_light: &'static str,
    pub bg_dark: &'static str,
    pub colors: &'static [&'static str],
}

pub const PALETTES: &[Palette] = &[
    Palette {
        name: "aurora",
        bg_light: "#e8f4f8",
        bg_dark: "#0a1628",
        colors: &["#00d4aa", "#7b68ee", "#ff6b9d", "#ffd93d", "#4ecdc4"],
    },
    Palette {
        name: "sunset",
        bg_light: "#fff5eb",
        bg_dark: "#1a0a14",
        colors: &["#ff6b35", "#f7931e", "#ffcd3c", "#c44569", "#e55039"],
    },
    Palette {
        name: "synthwave",
        bg_light: "#f0e6ff",
        bg_dark: "#0d0221",
        colors: &["#ff00ff", "#00ffff", "#bf00ff", "#ff1493", "#7b2cbf"],
    },
    Palette {
        name: "nord",
        bg_light: "#eceff4",
        bg_dark: "#2e3440",
        colors: &["#88c0d0", "#81a1c1", "#5e81ac", "#b48ead", "#a3be8c"],
    },
    Palette {
        name: "monochrome",
        bg_light: "#f5f5f5",
        bg_dark: "#1a1a1a",
        colors: &["#666666", "#888888", "#aaaaaa", "#444444", "#999999"],
    },
    Palette {
        name: "oceanic",
        bg_light: "#e6f7ff",
        bg_dark: "#001a33",
        colors: &["#0077b6", "#00b4d8", "#90e0ef", "#023e8a", "#48cae4"],
    },
    Palette {
        name: "neon",
        bg_light: "#f0fff0",
        bg_dark: "#0a0a0a",
        colors: &["#39ff14", "#ff073a", "#ffff00", "#00ffff", "#ff10f0"],
    },
    Palette {
        name: "pastel",
        bg_light: "#fffaf5",
        bg_dark: "#2d2a32",
        colors: &["#ffb5a7", "#fcd5ce", "#b8e0d2", "#d4a5a5", "#c9b1ff"],
    },
];

pub const PALETTE_COUNT: usize = PALETTES.len();

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Theme {
    #[default]
    Aurora,
    Sunset,
    Synthwave,
    Nord,
    Monochrome,
    Oceanic,
    Neon,
    Pastel,
}

impl Theme {
    pub fn from_name(name: &str) -> Option<Self> {
        let lower = name.to_ascii_lowercase();
        match lower.as_str() {
            "aurora" => Some(Theme::Aurora),
            "sunset" => Some(Theme::Sunset),
            "synthwave" => Some(Theme::Synthwave),
            "nord" => Some(Theme::Nord),
            "monochrome" => Some(Theme::Monochrome),
            "oceanic" => Some(Theme::Oceanic),
            "neon" => Some(Theme::Neon),
            "pastel" => Some(Theme::Pastel),
            _ => None,
        }
    }

    pub const fn index(self) -> usize {
        match self {
            Theme::Aurora => 0,
            Theme::Sunset => 1,
            Theme::Synthwave => 2,
            Theme::Nord => 3,
            Theme::Monochrome => 4,
            Theme::Oceanic => 5,
            Theme::Neon => 6,
            Theme::Pastel => 7,
        }
    }
}

pub fn palette_by_index(index: usize) -> &'static Palette {
    &PALETTES[index % PALETTE_COUNT]
}

pub fn palette_by_theme(theme: Theme) -> &'static Palette {
    palette_by_index(theme.index())
}
