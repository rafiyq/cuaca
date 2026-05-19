fn nice_ticks(min_val: f64, max_val: f64, steps: usize) -> Vec<f64> {
    if min_val == max_val {
        return vec![min_val; steps];
    }
    if steps < 2 {
        return vec![min_val];
    }

    let range = max_val - min_val;
    let rough_step = range / (steps as f64 - 1.0);

    if rough_step <= 0.0 {
        return vec![min_val; steps];
    }

    let magnitude = 10f64.powf(rough_step.log10().floor());
    let normalized = rough_step / magnitude;
    let nice_step = if normalized <= 1.5 {
        magnitude
    } else if normalized <= 3.0 {
        2.0 * magnitude
    } else if normalized <= 7.0 {
        5.0 * magnitude
    } else {
        10.0 * magnitude
    };

    let mut tick_start = (min_val / nice_step).floor() * nice_step;
    let tick_end = tick_start + nice_step * (steps as f64 - 1.0);

    if tick_end < max_val {
        tick_start -= nice_step;
    }

    (0..steps)
        .map(|i| {
            let v = tick_start + i as f64 * nice_step;
            (v / nice_step).round() * nice_step
        })
        .collect()
}

fn stretch_to_width(s: &str, target_w: usize) -> String {
    if s.is_empty() {
        return " ".repeat(target_w);
    }
    let n = s.chars().count();
    if n == target_w {
        return s.to_string();
    }
    let mut out = String::new();
    let base = target_w / n;
    let extra = target_w % n;
    for (i, ch) in s.chars().enumerate() {
        let repeat = base + if i < extra { 1 } else { 0 };
        out.push_str(&ch.to_string().repeat(repeat));
    }
    out
}

fn time_axis(times: &[String], width: usize) -> String {
    if times.is_empty() {
        return " ".repeat(width);
    }

    let n = times.len();
    if n == 1 {
        let mut axis = vec![' '; width];
        let t = &times[0];
        let pos = width / 2;
        for (j, ch) in t.chars().enumerate() {
            if pos + j < width {
                axis[pos + j] = ch;
            }
        }
        return axis.iter().collect();
    }

    let mut axis = vec![' '; width];

    for i in 0..n {
        let pos = i * width / n;
        let t = &times[i];
        for (j, ch) in t.chars().enumerate() {
            if pos + j < width {
                axis[pos + j] = ch;
            }
        }
    }

    for i in 1..n {
        let prev: u32 = times[i - 1].parse().unwrap_or(0);
        let curr: u32 = times[i].parse().unwrap_or(0);
        if curr <= prev && prev >= 18 {
            let pos = i * width / n;
            if pos > 0 && pos < width {
                axis[pos] = '│';
            }
        }
    }

    axis.iter().collect()
}

const SPARKLINE_CHARS: &[char] = &['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
const PANEL_GRAPH_W: usize = 24;

fn sparkline(values: &[f64]) -> String {
    if values.is_empty() {
        return String::new();
    }

    let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    if (max - min).abs() < f64::EPSILON {
        return SPARKLINE_CHARS[7]
            .to_string()
            .repeat(values.len().min(PANEL_GRAPH_W));
    }

    let range = max - min;
    values
        .iter()
        .take(PANEL_GRAPH_W)
        .map(|v| {
            let level = ((v - min) / range * 7.0).round() as usize;
            SPARKLINE_CHARS[level.clamp(0, 7)]
        })
        .collect()
}

pub fn temperature_panel(values: &[f64], times: &[String], height: usize) -> Vec<String> {
    if values.len() < 2 || height < 2 {
        return vec![];
    }

    let clipped: Vec<f64> = values.iter().take(PANEL_GRAPH_W).cloned().collect();
    let clipped_times: Vec<String> = times.iter().take(PANEL_GRAPH_W).cloned().collect();
    if clipped.is_empty() {
        return vec![];
    }

    let min_val = clipped.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_val = clipped.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = max_val - min_val;

    let braille_h = 4;

    let mapped: Vec<usize> = clipped
        .iter()
        .map(|v| {
            let y =
                braille_h as f64 - 1.0 - ((v - min_val) / range * (braille_h as f64 - 1.0)).round();
            (y.round() as usize).clamp(0, braille_h - 1)
        })
        .collect();

    let filled: Vec<usize> = {
        let mut result = Vec::new();
        for i in 0..mapped.len() {
            result.push(mapped[i]);
            if i + 1 < mapped.len() {
                let a = mapped[i] as i32;
                let b = mapped[i + 1] as i32;
                if (a - b).abs() > 1 {
                    result.push(((a + b) / 2).clamp(0, braille_h as i32 - 1) as usize);
                } else {
                    result.push(mapped[i + 1]);
                }
            }
        }
        result
    };

    let mut braille_chars = Vec::new();
    for i in (0..filled.len()).step_by(2) {
        let left_y = filled[i];
        let right_y = if i + 1 < filled.len() {
            filled[i + 1]
        } else {
            filled[i]
        };

        let dots = [
            (left_y == 0) as u8,
            (left_y == 1) as u8,
            (left_y == 2) as u8,
            (right_y == 0) as u8,
            (right_y == 1) as u8,
            (right_y == 2) as u8,
            (left_y == 3) as u8,
            (right_y == 3) as u8,
        ];

        let bits = dots[0]
            | (dots[1] << 1)
            | (dots[2] << 2)
            | (dots[3] << 3)
            | (dots[4] << 4)
            | (dots[5] << 5)
            | (dots[6] << 6)
            | (dots[7] << 7);

        let ch = char::from_u32(0x2800 + bits as u32).unwrap_or(' ');
        braille_chars.push(ch);
    }

    let braille_line: String = braille_chars.iter().collect();
    let stretched_braille = stretch_to_width(&braille_line, PANEL_GRAPH_W);

    let mut rows = Vec::new();

    for row_idx in 0..height {
        let factor = (height - 1 - row_idx) as f64 / (height - 1) as f64;
        let val_at_row = min_val + factor * range;
        let num_str = format!("{:.0}°", val_at_row.round() as i64);
        let label = format!("{:>5}", num_str);

        let mut row = label;
        row.push(' ');
        if row_idx == height / 2 {
            row.push_str(&stretched_braille);
        } else {
            row.push_str(&" ".repeat(PANEL_GRAPH_W));
        }
        rows.push(row);
    }

    let ta = time_axis(&clipped_times, PANEL_GRAPH_W);
    let mut time_row = "     ".to_string();
    time_row.push(' ');
    time_row.push_str(&ta);
    rows.push(time_row);

    rows
}

pub fn sparkline_panel(values: &[f64], times: &[String], height: usize) -> Vec<String> {
    if values.is_empty() || height < 2 {
        return vec![];
    }

    let sl = sparkline(values);
    let clipped_times: Vec<String> = times.iter().take(PANEL_GRAPH_W).cloned().collect();

    let stretched = stretch_to_width(&sl, PANEL_GRAPH_W);

    let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = max - min;

    let mut rows = Vec::new();
    for row_idx in 0..height {
        let factor = (height - 1 - row_idx) as f64 / (height - 1) as f64;
        let val_at_row = min + factor * range;
        let label = format!("{:>5.1}", val_at_row);

        let mut row = label;
        row.push(' ');
        if row_idx == height / 2 {
            row.push_str(&stretched);
        } else {
            row.push_str(&" ".repeat(PANEL_GRAPH_W));
        }
        rows.push(row);
    }

    let ta = time_axis(&clipped_times, PANEL_GRAPH_W);
    let mut time_row = "     ".to_string();
    time_row.push(' ');
    time_row.push_str(&ta);
    rows.push(time_row);

    rows
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sparkline_empty_values() {
        assert_eq!(sparkline(&[]), "");
    }

    #[test]
    fn sparkline_constant_values() {
        let result = sparkline(&[5.0, 5.0, 5.0]);
        assert_eq!(result, "███");
    }

    #[test]
    fn sparkline_increasing_values() {
        let result = sparkline(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]);
        assert_eq!(result, "▁▂▃▄▅▆▇█");
    }

    #[test]
    fn sparkline_caps_to_graph_width() {
        let values: Vec<f64> = (0..50).map(|i| i as f64).collect();
        let result = sparkline(&values);
        assert_eq!(result.chars().count(), PANEL_GRAPH_W);
    }

    #[test]
    fn temperature_panel_empty() {
        assert!(temperature_panel(&[], &[], 4).is_empty());
    }

    #[test]
    fn temperature_panel_single_value() {
        assert!(temperature_panel(&[25.0], &["20:00".to_string()], 4).is_empty());
    }

    #[test]
    fn temperature_panel_has_rows() {
        let rows = temperature_panel(
            &[25.0, 26.0, 27.0, 28.0, 29.0],
            &[
                "20".into(),
                "23".into(),
                "02".into(),
                "05".into(),
                "08".into(),
            ],
            4,
        );
        assert!(!rows.is_empty());
        assert_eq!(rows.len(), 4 + 1); // height + time axis
    }

    #[test]
    fn sparkline_panel_has_rows() {
        let rows = sparkline_panel(
            &[60.0, 70.0, 80.0, 90.0],
            &["20".into(), "23".into(), "02".into(), "05".into()],
            4,
        );
        assert!(!rows.is_empty());
        assert_eq!(rows.len(), 4 + 1);
    }

    #[test]
    fn sparkline_panel_min_max_labels() {
        let rows = sparkline_panel(
            &[10.0, 20.0, 30.0],
            &["20".into(), "23".into(), "02".into()],
            4,
        );
        let first_label = rows[0].trim();
        let last_label = rows[3].trim(); // PANEL_GRAPH_H-1 = 3
        assert!(first_label.starts_with("30"));
        assert!(last_label.starts_with("10"));
    }

    #[test]
    fn time_axis_shows_all_labels() {
        let times: Vec<String> = vec!["07", "10", "13", "16", "19", "22"]
            .into_iter()
            .map(String::from)
            .collect();
        let axis = time_axis(&times, 24);
        for hour in &times {
            assert!(
                axis.contains(hour),
                "Missing hour {} in axis: {}",
                hour,
                axis
            );
        }
    }
}
