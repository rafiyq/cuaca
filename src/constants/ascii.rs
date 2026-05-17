type AsciiIcon = [&'static str; 5];

const SUNNY: AsciiIcon = [
    "    \\   /    ",
    "     .-.     ",
    "  ― (   ) ―  ",
    "     `-´     ",
    "    /   \\    ",
];

const PARTLY_CLOUDY: AsciiIcon = [
    "   \\  /      ",
    " _ /\"\".-.   ",
    "   \\_(   ).  ",
    "   /(___(__) ",
    "             ",
];

const CLOUDY: AsciiIcon = [
    "             ",
    "     .--.    ",
    "  .-(    ).  ",
    " (___.__)__) ",
    "             ",
];

const FOG: AsciiIcon = [
    " _ - _ - _ - ",
    "  _ - _ - _  ",
    " _ - _ - _ - ",
    "             ",
    "             ",
];

const LIGHT_RAIN: AsciiIcon = [
    " _`/\"\".-.   ",
    "  ,\\_(   ).  ",
    "   /(___(__) ",
    "     ' ' ' ' ",
    "    ' ' ' '  ",
];

const HEAVY_RAIN: AsciiIcon = [
    " _`/\"\".-.   ",
    "  ,\\_(   ).  ",
    "   /(___(__) ",
    "   ' ' ' ' ' ",
    "  ' ' ' ' '  ",
];

const THUNDER: AsciiIcon = [
    "     .-.     ",
    "    (   ).   ",
    "   (___(__)  ",
    "     ' ' ' ' ",
    "    '   ' '  ",
];

const FALLBACK: AsciiIcon = [
    "             ",
    "     .-.     ",
    "    (   ).   ",
    "   (___(__)  ",
    "             ",
];

pub fn get_ascii_icon(code: u32) -> AsciiIcon {
    match code {
        0 | 1 => SUNNY,
        2 => PARTLY_CLOUDY,
        3 => CLOUDY,
        4 => CLOUDY,
        5 | 10 | 45 => FOG,
        60 | 61 => LIGHT_RAIN,
        63 | 80 => HEAVY_RAIN,
        95 | 97 => THUNDER,
        _ => FALLBACK,
    }
}
