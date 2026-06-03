pub fn format_wind_dir_icon(degrees: i64) -> &'static str {
    let dir = ((degrees % 360) as f64 / 45.0).round() as usize % 8;
    [
        "\u{2b06}\u{fe0f}",
        "\u{2197}\u{fe0f}",
        "\u{27a1}\u{fe0f}",
        "\u{2198}\u{fe0f}",
        "\u{2b07}\u{fe0f}",
        "\u{2199}\u{fe0f}",
        "\u{2b05}\u{fe0f}",
        "\u{2196}\u{fe0f}",
    ][dir]
}
