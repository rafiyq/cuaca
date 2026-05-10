pub const WEATHER_CODES: &[(u32, &str)] = &[
    (0, "\u{2600}\u{fe0f}"),
    (1, "\u{2600}\u{fe0f}"),
    (2, "\u{26c5}"),
    (3, "\u{2601}\u{fe0f}"),
    (4, "\u{1f325}\u{fe0f}"),
    (5, "\u{1f32b}\u{fe0f}"),
    (10, "\u{1f32b}\u{fe0f}"),
    (45, "\u{1f32b}\u{fe0f}"),
    (60, "\u{1f327}\u{fe0f}"),
    (61, "\u{1f327}\u{fe0f}"),
    (63, "\u{1f327}\u{fe0f}"),
    (80, "\u{1f327}\u{fe0f}"),
    (95, "\u{26c8}\u{fe0f}"),
    (97, "\u{26c8}\u{fe0f}"),
];

pub const WEATHER_CODES_NERD: &[(u32, &str)] = &[
    (0, "\u{F0319}"),
    (1, "\u{F0319}"),
    (2, "\u{F0315}"),
    (3, "\u{F0330}"),
    (4, "\u{F0310}"),
    (5, "\u{F0311}"),
    (10, "\u{F0311}"),
    (45, "\u{F0311}"),
    (60, "\u{F0317}"),
    (61, "\u{F0317}"),
    (63, "\u{F0316}"),
    (80, "\u{F0316}"),
    (95, "\u{F033E}"),
    (97, "\u{F033E}"),
];

pub const FALLBACK_ICON: &str = "\u{2600}\u{fe0f}";
pub const FALLBACK_ICON_NERD: &str = "\u{F0319}";

pub const ICON_PLACEHOLDER: &str = "{ICON}";
