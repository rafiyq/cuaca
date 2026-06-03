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
    "     .--.    ",
    "  .-(    ).  ",
    " (___.__)__) ",
    "             ",
];

const CODE_FOG: AsciiIcon = [
    "             ",
    " _ - _ - _ - ",
    "  _ - _ - _  ",
    " _ - _ - _ - ",
    "             ",
];

const CODE_HEAVY_RAIN: AsciiIcon = [
    "     .-.     ",
    "    (   ).   ",
    "   (___(__)  ",
    "  ‚ʻ‚ʻ‚ʻ‚ʻ   ",
    "  ‚ʻ‚ʻ‚ʻ‚ʻ   ",
];

const CODE_HEAVY_SHOWERS: AsciiIcon = [
    " _`/.-.    ",
    "  ,\\_(   ).  ",
    "   /(___(__) ",
    "   ‚ʻ‚ʻ‚ʻ‚ʻ  ",
    "   ‚ʻ‚ʻ‚ʻ‚ʻ  ",
];

const CODE_HEAVY_SNOW: AsciiIcon = [
    "     .-.     ",
    "    (   ).   ",
    "   (___(__)  ",
    "   * * * *   ",
    "  * * * *    ",
];

const CODE_HEAVY_SNOW_SHOWERS: AsciiIcon = [
    " _`/.-.    ",
    "  ,\\_(   ).  ",
    "   /(___(__) ",
    "    * * * *  ",
    "   * * * *   ",
];

const CODE_LIGHT_RAIN: AsciiIcon = [
    "     .-.     ",
    "    (   ).   ",
    "   (___(__)  ",
    "    ʻ ʻ ʻ ʻ  ",
    "   ʻ ʻ ʻ ʻ   ",
];

const CODE_LIGHT_SHOWERS: AsciiIcon = [
    " _`/.-.    ",
    "  ,\\_(   ).  ",
    "   /(___(__) ",
    "     ʻ ʻ ʻ ʻ ",
    "    ʻ ʻ ʻ ʻ  ",
];

const CODE_LIGHT_SLEET: AsciiIcon = [
    "     .-.     ",
    "    (   ).   ",
    "   (___(__)  ",
    "    ʻ * ʻ *  ",
    "   * ʻ * ʻ   ",
];

const CODE_LIGHT_SLEET_SHOWERS: AsciiIcon = [
    " _`/.-.    ",
    "  ,\\_(   ).  ",
    "   /(___(__) ",
    "     ʻ * ʻ * ",
    "    * ʻ * ʻ  ",
];

const CODE_LIGHT_SNOW: AsciiIcon = [
    "     .-.     ",
    "    (   ).   ",
    "   (___(__)  ",
    "    *  *  *  ",
    "   *  *  *   ",
];

const CODE_LIGHT_SNOW_SHOWERS: AsciiIcon = [
    " _`/.-.    ",
    "  ,\\_(   ).  ",
    "   /(___(__) ",
    "    *  *  *  ",
    "   *  *  *   ",
];

const CODE_PARTLY_CLOUDY: AsciiIcon = [
    "   \\__/      ",
    " __/  .-.    ",
    "   \\_(   ).  ",
    "   /(___(__) ",
    "             ",
];

const CODE_SUNNY: AsciiIcon = [
    "    \\ . /    ",
    "   - .-. -   ",
    "  ‒ (   ) ‒  ",
    "   . `-᾿ .   ",
    "    / ' \\    ",
];

const CODE_THUNDERY_HEAVY_RAIN: AsciiIcon = [
    "     .-.     ",
    "    (   ).   ",
    "   (___(__)  ",
    "  ‚ʻ⚡ʻ‚⚡‚ʻ   ",
    "  ‚ʻ‚ʻ⚡ʻ‚ʻ   ",
];

const CODE_THUNDERY_SHOWERS: AsciiIcon = [
    " _`/.-.    ",
    "  ,\\_(   ).  ",
    "   /(___(__) ",
    "    ⚡ʻ ʻ⚡ʻ ʻ ",
    "    ʻ ʻ ʻ ʻ  ",
];

const CODE_THUNDERY_SNOW_SHOWERS: AsciiIcon = [
    " _`/.-.    ",
    "  ,\\_(   ).  ",
    "   /(___(__) ",
    "     *⚡ *⚡ * ",
    "    *  *  *  ",
];

const CODE_VERY_CLOUDY: AsciiIcon = [
    "             ",
    "     .--.    ",
    "  .-(    ).  ",
    " (___.__)__) ",
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
