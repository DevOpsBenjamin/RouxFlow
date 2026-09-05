use rouxflow_bitboard::BitCube;

pub fn colored_sticker(cube: &BitCube, bit_idx: usize) -> String {
    const RESET: &str = "\x1b[0m";
    const COLORS: [(&str, char); 6] = [
        ("\x1b[97;1m", 'W'),     // White (bright bold)
        ("\x1b[93;1m", 'Y'),     // Yellow (bright bold)
        ("\x1b[32;1m", 'G'),     // Green (bold)
        ("\x1b[34;1m", 'B'),     // Blue (bold)
        ("\x1b[31;1m", 'R'),     // Red (bold)
        ("\x1b[38;5;208m", 'O'), // Orange (256-color)
    ];
    let c = cube.get_color_at(bit_idx);
    format!("{}{}{}", COLORS[c].0, COLORS[c].1, RESET)
}

fn cube_face_row(cube: &BitCube, face_offset: usize, row: usize) -> String {
    let mut s = String::new();
    for col in 0..3 {
        s.push_str(&colored_sticker(cube, face_offset + row * 3 + col));
        s.push(' ');
    }
    s
}

/// Render a cube to 9 lines (with ANSI colors). Each line has the same visual width (24 chars).
pub fn cube_to_lines(cube: &BitCube) -> Vec<String> {
    let pad = "      "; // 6 spaces = L face width (3 stickers × 2 chars)
    let trail = "            "; // 12 spaces (pads U/D rows to 24 visual width)
    let mut lines = Vec::with_capacity(9);

    // U face (rows 0-2)
    for row in 0..3 {
        lines.push(format!("{}{}{}", pad, cube_face_row(cube, 0, row), trail));
    }
    // Middle band: L F R B (rows 0-2)
    for row in 0..3 {
        lines.push(format!(
            "{}{}{}{}",
            cube_face_row(cube, 36, row), // L
            cube_face_row(cube, 18, row), // F
            cube_face_row(cube, 9, row),  // R
            cube_face_row(cube, 45, row), // B
        ));
    }
    // D face (rows 0-2)
    for row in 0..3 {
        lines.push(format!("{}{}{}", pad, cube_face_row(cube, 27, row), trail));
    }
    lines
}

pub fn print_cubes_side_by_side(cubes: &[(&BitCube, &str)]) {
    let all_lines: Vec<Vec<String>> = cubes.iter().map(|(c, _)| cube_to_lines(c)).collect();

    // Header
    let header: Vec<String> = cubes
        .iter()
        .map(|(_, label)| format!("{:^24}", label))
        .collect();
    println!("{}", header.join("  |  "));

    // Rows
    for row in 0..9 {
        let parts: Vec<&str> = all_lines.iter().map(|lines| lines[row].as_str()).collect();
        println!("{}", parts.join("  |  "));
    }
    println!();
}
