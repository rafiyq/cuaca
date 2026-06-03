use crate::color;

pub fn colorize_temp_panel(rows: &mut [String], temps: &[f64]) {
    let min_t = temps.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_t = temps.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    for i in 0..rows.len() {
        let rev_i = rows.len() - 1 - i;
        let temp_at_row = if rows.len() <= 1 {
            25.0
        } else {
            min_t + (max_t - min_t) * (rev_i as f64 / (rows.len() - 1) as f64)
        };
        rows[i] = color::temp_line(&rows[i], temp_at_row as i64);
    }
}

pub fn colorize_spark_panel(rows: &mut [String], color_fn: fn(&str) -> String) {
    for row in rows {
        *row = color_fn(row);
    }
}
