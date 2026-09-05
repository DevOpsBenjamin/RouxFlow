/// Generate a random 20-move scramble string.
/// `rng` must return a value in [0.0, 1.0).
pub fn generate_scramble(rng: &mut impl FnMut() -> f64) -> String {
    let faces = ["U", "R", "F", "D", "L", "B"];
    let mods = ["", "'", "2"];
    let mut result = Vec::with_capacity(20);
    let mut last: Option<usize> = None;
    let mut second_last: Option<usize> = None;
    for _ in 0..20 {
        loop {
            let fi = (rng() * 6.0) as usize % 6;
            if last == Some(fi) { continue; }
            // Prevent 3 consecutive on same axis (U/D=0/3, R/L=1/4, F/B=2/5)
            if let (Some(l), Some(sl)) = (last, second_last) {
                if fi % 3 == l % 3 && l % 3 == sl % 3 { continue; }
            }
            let mi = (rng() * 3.0) as usize % 3;
            result.push(format!("{}{}", faces[fi], mods[mi]));
            second_last = last;
            last = Some(fi);
            break;
        }
    }
    result.join(" ")
}
