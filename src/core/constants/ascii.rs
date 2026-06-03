#![allow(dead_code)]

type AsciiIcon = [&'static str; 5];

const CODE_UNKNOWN: AsciiIcon = [
    "    .-.      ",
    "     __)     ",
    "    (        ",
    "     `-᾿     ",
    "      •      ",
];

const CODE_CLOUDY: AsciiIcon = [
    "             ",
    "\x1b[38;5;250m     .--.    \x1b[0m",
    "\x1b[38;5;250m  .-(    ).  \x1b[0m",
    "\x1b[38;5;250m (___.__)__) \x1b[0m",
    "             ",
];

const CODE_FOG: AsciiIcon = [
    "             ",
    "\x1b[38;5;251m _ - _ - _ - \x1b[0m",
    "\x1b[38;5;251m  _ - _ - _  \x1b[0m",
    "\x1b[38;5;251m _ - _ - _ - \x1b[0m",
    "             ",
];

const CODE_HEAVY_RAIN: AsciiIcon = [
    "\x1b[38;5;244;1m     .-.     \x1b[0m",
    "\x1b[38;5;244;1m    (   ).   \x1b[0m",
    "\x1b[38;5;244;1m   (___(__)  \x1b[0m",
    "\x1b[38;5;33;1m  ‚ʻ‚ʻ‚ʻ‚ʻ   \x1b[0m",
    "\x1b[38;5;33;1m  ‚ʻ‚ʻ‚ʻ‚ʻ   \x1b[0m",
];

const CODE_HEAVY_SHOWERS: AsciiIcon = [
    "\x1b[38;5;226m _`/\x1b[38;5;244;1m.-.    \x1b[0m",
    "\x1b[38;5;226m  ,\\_\x1b[38;5;244;1m(   ).  \x1b[0m",
    "\x1b[38;5;226m   /\x1b[38;5;244;1m(___(__) \x1b[0m",
    "\x1b[38;5;33;1m   ‚ʻ‚ʻ‚ʻ‚ʻ  \x1b[0m",
    "\x1b[38;5;33;1m   ‚ʻ‚ʻ‚ʻ‚ʻ  \x1b[0m",
];

const CODE_HEAVY_SNOW: AsciiIcon = [
    "\x1b[38;5;244;1m     .-.     \x1b[0m",
    "\x1b[38;5;244;1m    (   ).   \x1b[0m",
    "\x1b[38;5;244;1m   (___(__)  \x1b[0m",
    "\x1b[38;5;255;1m   * * * *   \x1b[0m",
    "\x1b[38;5;255;1m  * * * *    \x1b[0m",
];

const CODE_HEAVY_SNOW_SHOWERS: AsciiIcon = [
    "\x1b[38;5;226m _`/\x1b[38;5;244;1m.-.    \x1b[0m",
    "\x1b[38;5;226m  ,\\_\x1b[38;5;244;1m(   ).  \x1b[0m",
    "\x1b[38;5;226m   /\x1b[38;5;244;1m(___(__) \x1b[0m",
    "\x1b[38;5;255;1m    * * * *  \x1b[0m",
    "\x1b[38;5;255;1m   * * * *   \x1b[0m",
];

const CODE_LIGHT_RAIN: AsciiIcon = [
    "\x1b[38;5;250m     .-.     \x1b[0m",
    "\x1b[38;5;250m    (   ).   \x1b[0m",
    "\x1b[38;5;250m   (___(__)  \x1b[0m",
    "\x1b[38;5;111m    ʻ ʻ ʻ ʻ  \x1b[0m",
    "\x1b[38;5;111m   ʻ ʻ ʻ ʻ   \x1b[0m",
];

const CODE_LIGHT_SHOWERS: AsciiIcon = [
    "\x1b[38;5;226m _`/\x1b[38;5;250m.-.    \x1b[0m",
    "\x1b[38;5;226m  ,\\_\x1b[38;5;250m(   ).  \x1b[0m",
    "\x1b[38;5;226m   /\x1b[38;5;250m(___(__) \x1b[0m",
    "\x1b[38;5;111m     ʻ ʻ ʻ ʻ \x1b[0m",
    "\x1b[38;5;111m    ʻ ʻ ʻ ʻ  \x1b[0m",
];

const CODE_LIGHT_SLEET: AsciiIcon = [
    "\x1b[38;5;250m     .-.     \x1b[0m",
    "\x1b[38;5;250m    (   ).   \x1b[0m",
    "\x1b[38;5;250m   (___(__)  \x1b[0m",
    "\x1b[38;5;111m    ʻ \x1b[38;5;255m*\x1b[38;5;111m ʻ \x1b[38;5;255m*  \x1b[0m",
    "\x1b[38;5;255m   *\x1b[38;5;111m ʻ \x1b[38;5;255m*\x1b[38;5;111m ʻ   \x1b[0m",
];

const CODE_LIGHT_SLEET_SHOWERS: AsciiIcon = [
    "\x1b[38;5;226m _`/\x1b[38;5;250m.-.    \x1b[0m",
    "\x1b[38;5;226m  ,\\_\x1b[38;5;250m(   ).  \x1b[0m",
    "\x1b[38;5;226m   /\x1b[38;5;250m(___(__) \x1b[0m",
    "\x1b[38;5;111m     ʻ \x1b[38;5;255m*\x1b[38;5;111m ʻ \x1b[38;5;255m* \x1b[0m",
    "\x1b[38;5;255m    *\x1b[38;5;111m ʻ \x1b[38;5;255m*\x1b[38;5;111m ʻ  \x1b[0m",
];

const CODE_LIGHT_SNOW: AsciiIcon = [
    "\x1b[38;5;250m     .-.     \x1b[0m",
    "\x1b[38;5;250m    (   ).   \x1b[0m",
    "\x1b[38;5;250m   (___(__)  \x1b[0m",
    "\x1b[38;5;255m    *  *  *  \x1b[0m",
    "\x1b[38;5;255m   *  *  *   \x1b[0m",
];

const CODE_LIGHT_SNOW_SHOWERS: AsciiIcon = [
    "\x1b[38;5;226m _`/\x1b[38;5;250m.-.    \x1b[0m",
    "\x1b[38;5;226m  ,\\_\x1b[38;5;250m(   ).  \x1b[0m",
    "\x1b[38;5;226m   /\x1b[38;5;250m(___(__) \x1b[0m",
    "\x1b[38;5;255m     *  *  * \x1b[0m",
    "\x1b[38;5;255m    *  *  *  \x1b[0m",
];

const CODE_PARTLY_CLOUDY: AsciiIcon = [
    "\x1b[38;5;226m   \\__/\x1b[0m      ",
    "\x1b[38;5;226m __/  \x1b[38;5;250m.-.    \x1b[0m",
    "\x1b[38;5;226m   \\_\x1b[38;5;250m(   ).  \x1b[0m",
    "\x1b[38;5;226m   /\x1b[38;5;250m(___(__) \x1b[0m",
    "             ",
];

const CODE_SUNNY: AsciiIcon = [
    "\x1b[38;5;226m    \\ . /    \x1b[0m",
    "\x1b[38;5;226m   - .-. -   \x1b[0m",
    "\x1b[38;5;226m  ‒ (   ) ‒  \x1b[0m",
    "\x1b[38;5;226m   . `-᾿ .   \x1b[0m",
    "\x1b[38;5;226m    / ' \\    \x1b[0m",
];

const CODE_THUNDERY_HEAVY_RAIN: AsciiIcon = [
    "\x1b[38;5;244;1m     .-.     \x1b[0m",
    "\x1b[38;5;244;1m    (   ).   \x1b[0m",
    "\x1b[38;5;244;1m   (___(__)  \x1b[0m",
    "\x1b[38;5;33;1m  ‚ʻ\x1b[38;5;228;5m⚡\x1b[38;5;33;25mʻ‚\x1b[38;5;228;5m⚡\x1b[38;5;33;25m‚ʻ   \x1b[0m",
    "\x1b[38;5;33;1m  ‚ʻ‚ʻ\x1b[38;5;228;5m⚡\x1b[38;5;33;25mʻ‚ʻ   \x1b[0m",
];

const CODE_THUNDERY_SHOWERS: AsciiIcon = [
    "\x1b[38;5;226m _`/\x1b[38;5;250m.-.    \x1b[0m",
    "\x1b[38;5;226m  ,\\_\x1b[38;5;250m(   ).  \x1b[0m",
    "\x1b[38;5;226m   /\x1b[38;5;250m(___(__) \x1b[0m",
    "\x1b[38;5;228;5m    ⚡\x1b[38;5;111;25mʻ ʻ\x1b[38;5;228;5m⚡\x1b[38;5;111;25mʻ ʻ \x1b[0m",
    "\x1b[38;5;111m    ʻ ʻ ʻ ʻ  \x1b[0m",
];

const CODE_THUNDERY_SNOW_SHOWERS: AsciiIcon = [
    "\x1b[38;5;226m _`/\x1b[38;5;250m.-.    \x1b[0m",
    "\x1b[38;5;226m  ,\\_\x1b[38;5;250m(   ).  \x1b[0m",
    "\x1b[38;5;226m   /\x1b[38;5;250m(___(__) \x1b[0m",
    "\x1b[38;5;255m     *\x1b[38;5;228;5m⚡\x1b[38;5;255;25m *\x1b[38;5;228;5m⚡\x1b[38;5;255;25m * \x1b[0m",
    "\x1b[38;5;255m    *  *  *  \x1b[0m",
];

const CODE_VERY_CLOUDY: AsciiIcon = [
    "             ",
    "\x1b[38;5;244;1m     .--.    \x1b[0m",
    "\x1b[38;5;244;1m  .-(    ).  \x1b[0m",
    "\x1b[38;5;244;1m (___.__)__) \x1b[0m",
    "             ",
];

pub fn get_ascii_icon(code: u32) -> AsciiIcon {
    match code {
        0 | 1 => CODE_SUNNY,
        2 => CODE_PARTLY_CLOUDY,
        3 => CODE_CLOUDY,
        4 => CODE_VERY_CLOUDY,
        5 | 10 | 45 => CODE_FOG,
        60 | 61 => CODE_LIGHT_RAIN,
        63 | 80 => CODE_HEAVY_RAIN,
        95 | 97 => CODE_THUNDERY_HEAVY_RAIN,
        _ => CODE_UNKNOWN,
    }
}
