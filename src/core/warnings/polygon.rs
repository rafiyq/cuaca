pub(super) fn parse_polygon(s: &str) -> Vec<(f64, f64)> {
    s.split_whitespace()
        .filter_map(|coord| {
            let parts: Vec<&str> = coord.split(',').collect();
            if parts.len() == 2 {
                let lat = parts[0].parse::<f64>().ok()?;
                let lon = parts[1].parse::<f64>().ok()?;
                Some((lat, lon))
            } else {
                None
            }
        })
        .collect()
}

pub(super) fn point_in_polygon(point: (f64, f64), poly: &[(f64, f64)]) -> bool {
    let (x, y) = point;
    let mut inside = false;
    let mut j = poly.len() - 1;
    for i in 0..poly.len() {
        let (xi, yi) = poly[i];
        let (xj, yj) = poly[j];
        let intersect = ((yi > y) != (yj > y)) && (x < (xj - xi) * (y - yi) / (yj - yi) + xi);
        if intersect {
            inside = !inside;
        }
        j = i;
    }
    inside
}
