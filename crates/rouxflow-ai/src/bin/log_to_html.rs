use rouxflow_bitboard::{BitCube, Move};
use rouxflow_core::cube::{facelet::Color, Orientation};
use rouxflow_core::telemetry::{DebugTrace, SimpleSolveEvent};
use std::collections::BTreeMap;
use std::fs;

#[derive(Debug, Default)]
struct TimelineRow {
    pass1_move: Option<String>,
    pass2_state: Option<String>,
    pass2_gyro_eff: Option<String>,
    pass2_gyro_runs: Vec<String>,
    pass3_rot: Vec<String>,
    pass4_move: Option<(String, bool)>, // label, is_rot
    pass4_state: Option<String>,
}

fn generate_html(trace: &DebugTrace, title: &str) -> String {
    // We group by "time" (approximate or ordered index).
    // Using BTreeMap with ordered float keys might be tricky due to float precision,
    // so we'll convert to milliseconds rounded to nearest int as the key for alignment.
    let mut timeline: BTreeMap<i64, TimelineRow> = BTreeMap::new();

    for p1 in &trace.pass1_moves {
        let ms = (p1.t * 1000.0) as i64;
        let row = timeline.entry(ms).or_default();
        row.pass1_move = Some(p1.merged_move.clone());
    }

    for p2 in &trace.pass2_states {
        let ms = (p2.t * 1000.0) as i64;
        let row = timeline.entry(ms).or_default();
        row.pass2_state = Some(p2.cube_state.clone());
        row.pass2_gyro_eff = Some(p2.active_gyro_window.clone());

        for run in &p2.gyro_runs {
            let run_ms = (run.t * 1000.0) as i64;
            let run_row = timeline.entry(run_ms).or_default();
            run_row.pass2_gyro_runs.push(run.label.clone());
        }
    }

    for p3 in &trace.pass3_rotations {
        let ms = (p3.t * 1000.0) as i64;
        let row = timeline.entry(ms).or_default();
        let insp = if p3.is_inspection {
            " (Inspection)"
        } else {
            ""
        };
        row.pass3_rot.push(format!(
            "{} ({} -> {}){}",
            p3.rotation_label, p3.from_orient, p3.to_orient, insp
        ));
    }

    if let Some(clean) = &trace.clean_replay {
        let mut cube = BitCube::new_solved();

        if !trace.scramble.is_empty() {
            for m_str in trace.scramble.split_whitespace() {
                cube.apply_move(m_str);
            }
        }

        let bitcube_home = Orientation {
            top: Color::White,
            front: Color::Green,
        };
        let initial_rot = rouxflow_ai::gyro_analyzer::math::detect_rotation(
            bitcube_home,
            clean.initial_orientation,
        );
        for rot_part in initial_rot.split_whitespace() {
            cube.apply_move(rot_part);
        }

        for event in &clean.timeline {
            let (ms, label, is_rot) = match event {
                SimpleSolveEvent::Move { t, m } => {
                    cube.apply_move_enum(*m);
                    ((t * 1000.0) as i64, m.as_str().to_string(), false)
                }
                SimpleSolveEvent::Rotation { t, axis } => {
                    cube.apply_move_enum(Move::Rotate(*axis));
                    ((t * 1000.0) as i64, axis.as_str().to_string(), true)
                }
            };
            let row = timeline.entry(ms).or_default();
            row.pass4_move = Some((label, is_rot));
            row.pass4_state = Some(cube.to_html_string());
        }
    }

    let mut html = String::new();
    html.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
    html.push_str("<title>Debug Replay: ");
    html.push_str(title);
    html.push_str("</title>\n");
    html.push_str(r#"
    <style>
        body { font-family: sans-serif; background: #1e1e1e; color: #ddd; margin: 0; padding: 20px; }
        h1 { color: #fff; border-bottom: 2px solid #555; padding-bottom: 10px; }
        table { width: 100%; border-collapse: collapse; margin-top: 20px; }
        th, td { border: 1px solid #444; padding: 10px; text-align: left; vertical-align: top; }
        th { background: #333; color: #fff; position: sticky; top: 0; z-index: 10; }
        tr:nth-child(even) { background: #252525; }
        .time { font-family: monospace; color: #88ccff; }
        .move { font-weight: bold; color: #ffeb3b; font-size: 1.2em; }
        .state { font-family: monospace; white-space: pre; font-size: 14px; line-height: 1.2; letter-spacing: 2px; }
        .rot { background: #b71c1c; color: #fff; padding: 2px 6px; border-radius: 4px; font-weight: bold; }
        .gyro { color: #ccc; font-style: italic; margin-bottom: 8px; }
        .gyro-run { font-family: monospace; color: #a1b56c; font-size: 0.9em; }
        .gyro-noise { color: #777; font-style: italic; }
        .cW { font-weight: bold; color: #ffffff; }
        .cY { font-weight: bold; color: #ffff00; }
        .cG { font-weight: bold; color: #00ff00; }
        .cB { font-weight: bold; color: #0088ff; }
        .cR { font-weight: bold; color: #ff3333; }
        .cO { font-weight: bold; color: #ff8800; }
    </style>
    "#);
    html.push_str("</head>\n<body>\n");
    html.push_str(&format!("<h1>{}</h1>\n", title));
    if !trace.scramble.is_empty() {
        html.push_str(&format!(
            "<h3 style='color:#b71c1c; margin-top:-10px; margin-bottom:20px'>Scramble: {}</h3>\n",
            trace.scramble
        ));
    }

    html.push_str("<table>\n");
    html.push_str("<tr><th>Time (s)</th><th>Pass 1: Move</th><th>Pass 2: Gyro Runs</th><th>Pass 2: Effective</th><th>Pass 2: State</th><th>Pass 3: Rotations</th><th>Pass 4: Clean Replay</th></tr>\n");

    for (ms, row) in timeline {
        let t_sec = (ms as f64) / 1000.0;
        html.push_str("<tr>\n");
        html.push_str(&format!("<td class='time'>{:.3}s</td>\n", t_sec));

        // Pass 1
        html.push_str("<td>");
        if let Some(m) = &row.pass1_move {
            html.push_str(&format!("<span class='move'>{}</span>", m));
        }
        html.push_str("</td>\n");

        // Pass 2: Gyro Runs
        html.push_str("<td>");
        for run in &row.pass2_gyro_runs {
            if run.contains("noise") {
                html.push_str(&format!("<div class='gyro-run gyro-noise'>{}</div>", run));
            } else {
                html.push_str(&format!("<div class='gyro-run'>{}</div>", run));
            }
        }
        html.push_str("</td>\n");

        // Pass 2: Effective Gyro
        html.push_str("<td>");
        if let Some(g) = &row.pass2_gyro_eff {
            html.push_str(&format!(
                "<div class='gyro'><strong>Effective:</strong> {}</div>",
                g
            ));
        }
        html.push_str("</td>\n");

        // Pass 2: State
        html.push_str("<td>");
        if let Some(s) = &row.pass2_state {
            html.push_str(&format!("<div class='state'>{}</div>", s));
        }
        html.push_str("</td>\n");

        // Pass 3
        html.push_str("<td>");
        for r in &row.pass3_rot {
            html.push_str(&format!(
                "<div class='rot' style='margin-bottom:4px'>{}</div>",
                r
            ));
        }
        html.push_str("</td>\n");

        // Pass 4
        html.push_str("<td>");
        if let Some((m, is_rot)) = &row.pass4_move {
            if *is_rot {
                html.push_str(&format!(
                    "<div class='move rot' style='display:inline-block'>{}</div>",
                    m
                ));
            } else {
                html.push_str(&format!(
                    "<div class='move' style='color:#a8ff60'>{}</div>",
                    m
                ));
            }
        }
        if let Some(s) = &row.pass4_state {
            html.push_str(&format!("<div class='state'>{}</div>", s));
        }
        html.push_str("</td>\n");

        html.push_str("</tr>\n");
    }

    html.push_str("</table>\n");

    html.push_str("</body>\n</html>");
    html
}

fn main() {
    for solve_id in &["1", "2"] {
        let input_file = format!("solve{}_trace.json", solve_id);
        let output_file = format!("solve{}_debug.html", solve_id);

        if let Ok(content) = fs::read_to_string(&input_file) {
            match serde_json::from_str::<DebugTrace>(&content) {
                Ok(trace) => {
                    let html = generate_html(&trace, &format!("Solve {} Trace Analysis", solve_id));
                    fs::write(&output_file, html).unwrap();
                    println!("Generated {}", output_file);
                }
                Err(e) => {
                    println!("Failed to parse {}: {}", input_file, e);
                }
            }
        } else {
            println!("Could not read {}", input_file);
        }
    }
}
