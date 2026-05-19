pub fn write_f32(out: &mut String, v: f32) {
    let rounded = (v * 10.0).round() / 10.0;
    if rounded.fract().abs() < 0.05 {
        let i = rounded as i32;
        out.push_str(&i.to_string());
    } else {
        out.push_str(&format!("{rounded:.1}"));
    }
}
