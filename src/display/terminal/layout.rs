pub fn render_row(
    out: &mut String,
    title1: &str,
    panel1: &[String],
    title2: &str,
    panel2: &[String],
    title3: &str,
    panel3: &[String],
) {
    let title_field = |t: &str| -> String {
        // Truncate safely to 24 characters (avoid splitting multi-byte)
        let t_truncated = if t.chars().count() > 24 {
            t.chars().take(24).collect()
        } else {
            t.to_string()
        };
        let pad_left = 6;
        let graph_w = 24;
        let total_w = pad_left + graph_w;
        let title_len = t_truncated.len();
        let pad = if title_len >= graph_w {
            0
        } else {
            (graph_w - title_len) / 2
        };
        let mut field = " ".repeat(pad_left + pad);
        field.push_str(&t_truncated);
        let remaining = total_w - field.len();
        if remaining > 0 {
            field.push_str(&" ".repeat(remaining));
        }
        field
    };

    out.push_str(&format!(
        "{}  {}  {}\n",
        title_field(title1),
        title_field(title2),
        title_field(title3)
    ));

    let max_h = panel1.len().max(panel2.len()).max(panel3.len());
    for r in 0..max_h {
        let l1 = panel1
            .get(r)
            .map(|s| format!("{:<30}", s))
            .unwrap_or_else(|| " ".repeat(30));
        let l2 = panel2
            .get(r)
            .map(|s| format!("{:<30}", s))
            .unwrap_or_else(|| " ".repeat(30));
        let l3 = panel3
            .get(r)
            .map(|s| format!("{:<30}", s))
            .unwrap_or_else(|| " ".repeat(30));
        out.push_str(&format!("{}  {}  {}\n", l1, l2, l3));
    }
}
