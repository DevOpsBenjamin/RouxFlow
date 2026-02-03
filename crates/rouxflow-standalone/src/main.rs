use three_d::*;
use rouxflow_core::cube::CubeState;
use rouxflow_render::RenderState;
use std::time::{Duration, Instant};

#[derive(PartialEq, Clone, Copy, Debug)]
enum RouxPhase { FB, SB, CMLL, LSE_EO, LSE_ULUR, LSE_L4E, DONE }


fn main() {
    // Create native window
    let window = Window::new(WindowSettings {
        title: "RouxFlow Standalone Test".to_string(),
        max_size: Some((480, 480)),
        ..Default::default()
    }).unwrap();

    // Get GL context from window
    let context = window.gl();
    
    // Initialize render state (manages camera, controls, models)
    let mut render_state = RenderState::new(&context, window.viewport());
    
    // Initialize cube state (solved)
    let mut cube_state = CubeState::new();
    
    // Parse CLI arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: rouxflow-standalone \"<scramble>\" \"<solve>\" [--debug]");
        eprintln!("Example: rouxflow-standalone \"R U R' U'\" \"x2 y U R U' R'\" --debug");
        std::process::exit(1);
    }
    
    let scramble_moves: Vec<String> = args[1].split_whitespace().map(|s| s.to_string()).collect();
    let solve_moves: Vec<String> = args[2].split_whitespace().map(|s| s.to_string()).collect();
    let debug_mode = args.get(3).map(|s| s == "--debug").unwrap_or(false);

    // Timing and Progress
    let mut last_move = Instant::now();
    let mut move_idx = 0;
    let mut is_solving = false;
    
    // Roux Phase Tracking
    let mut current_phase = RouxPhase::FB;
    let mut phase_moves: Vec<(String, Vec<String>)> = vec![
        ("FB".to_string(), Vec::new()),
        ("SB".to_string(), Vec::new()),
        ("CMLL".to_string(), Vec::new()),
        ("LSE (EO)".to_string(), Vec::new()),
        ("LSE (ULUR)".to_string(), Vec::new()),
        ("LSE (Finish)".to_string(), Vec::new()),
    ];
    


    if !scramble_moves.is_empty() {
        println!("Scramble provided: {} moves", scramble_moves.len());
    }
    if !solve_moves.is_empty() {
        println!("Solve provided: {} moves", solve_moves.len());
    }

    // Main render loop
    window.render_loop(move |mut frame_input| {
        render_state.set_viewport(frame_input.viewport);
        render_state.handle_events(&mut frame_input.events);
        
        let now = Instant::now();
        
        // 1. Play Scramble (Fast: 200ms between moves, 150ms animation)
        if !is_solving && move_idx < scramble_moves.len() {
            if last_move.elapsed() >= Duration::from_millis(200) {
                let m = &scramble_moves[move_idx];
                cube_state.apply_move(m);
                render_state.trigger_move_anim(m, 0.15);

                move_idx += 1;
                last_move = now;
                if move_idx == scramble_moves.len() {
                    println!("Scramble complete. Starting solve playback...");
                    is_solving = true;
                    move_idx = 0;
                    last_move = now + Duration::from_secs(1);
                }
            }
        } 
        // 2. Play Solve (Snappy: 150ms between moves, 100ms animation)
        else if is_solving && move_idx < solve_moves.len() {
            if last_move.elapsed() >= Duration::from_millis(150) {
                let m = &solve_moves[move_idx];
                cube_state.apply_move(m);
                render_state.trigger_move_anim(m, 0.1);

                move_idx += 1;
                last_move = now;

                // Track move for analysis
                track_move_phase(&mut current_phase, &mut phase_moves, &cube_state, m, move_idx, solve_moves.len(), debug_mode);

                if move_idx == solve_moves.len() {
                    print_solve_analysis(&cube_state, &phase_moves);
                }
            }
        }


        // Render frame
        render_state.update_cube_state(&cube_state.get_facelets(), None);
        render_state.render_frame(&frame_input.screen(), frame_input.elapsed_time as f32 / 1000.0);
        
        FrameOutput::default()
    });
}

fn track_move_phase(
    current_phase: &mut RouxPhase, 
    phase_moves: &mut Vec<(String, Vec<String>)>, 
    cube: &CubeState, 
    m: &str, 
    idx: usize, 
    total: usize,
    debug: bool
) {
    let phase_idx = match current_phase {
        RouxPhase::FB => 0,
        RouxPhase::SB => 1,
        RouxPhase::CMLL => 2,
        RouxPhase::LSE_EO => 3,
        RouxPhase::LSE_ULUR => 4,
        _ => 5,
    };
    phase_moves[phase_idx].1.push(m.to_string());

    if debug {
        let fb = if cube.is_fb_solved() { "✅" } else { "❌" };
        let sb = if cube.is_sb_solved() { "✅" } else { "❌" };
        let cmll = if cube.is_cmll_solved() { "✅" } else { "❌" };
        let bad_edges = cube.count_bad_edges();
        let ulur = if cube.is_ul_ur_placed() { "✅" } else { "❌" };
        let l4e = if cube.is_l4e_solved() { "✅" } else { "❌" };
        
        println!("[Solve {:2}/{:2}] {:<3} | FB: {} | SB: {} | CMLL: {} | EO: {} bad | ULUR: {} | L4E: {}", 
            idx, total, m, fb, sb, cmll, bad_edges, ulur, l4e);
    }

    // Transition logic
    let bad_edges = cube.count_bad_edges();
    if *current_phase == RouxPhase::FB && cube.is_fb_solved() {
        *current_phase = RouxPhase::SB;
        println!(">>> Progress: First Block Finished [Solve {:2}/{:2}]", idx, total);
    } else if *current_phase == RouxPhase::SB && cube.is_sb_solved() {
        *current_phase = RouxPhase::CMLL;
        println!(">>> Progress: Second Block Finished [Solve {:2}/{:2}]", idx, total);
    } else if *current_phase == RouxPhase::CMLL && cube.is_cmll_solved() {
        *current_phase = RouxPhase::LSE_EO;
        println!(">>> Progress: CMLL Finished [Solve {:2}/{:2}]", idx, total);
    } else if *current_phase == RouxPhase::LSE_EO && bad_edges == 0 {
        *current_phase = RouxPhase::LSE_ULUR;
        println!(">>> Progress: LSE EO Finished [Solve {:2}/{:2}]", idx, total);
    } else if *current_phase == RouxPhase::LSE_ULUR && cube.is_ul_ur_placed() {
        *current_phase = RouxPhase::LSE_L4E;
        println!(">>> Progress: LSE UL/UR Finished [Solve {:2}/{:2}]", idx, total);
    } else if *current_phase == RouxPhase::LSE_L4E && cube.is_l4e_solved() {
        *current_phase = RouxPhase::DONE;
        println!(">>> Progress: Solve Finished [Solve {:2}/{:2}]", idx, total);
    }
}

fn print_solve_analysis(cube: &CubeState, phase_moves: &[(String, Vec<String>)]) {
    if !cube.is_solved() {
        println!("\n⚠️  WARNING: CUBE IS NOT SOLVED AT THE END!");
    } else {
        println!("\n✅ SUCCESS: CUBE IS FULLY SOLVED!");
    }
    println!("Final cube representation:");
    cube.dump_debug();

    println!("\n===== ROUX SOLVE ANALYSIS =====");
    
    let mut total_moves = 0;
    for (name, moves) in phase_moves {
        if !moves.is_empty() {
            println!("{:<15}: {:2} moves -> {}", name, moves.len(), moves.join(" "));
            total_moves += moves.len();
        }
    }
    println!("-------------------------------");
    println!("Total Moves: {}", total_moves);
    println!();
    
    println!("=== ROUX STYLE ANALYSIS ===");
    let mut penalties = 0;
    let mut issues: Vec<String> = Vec::new();
    
    // FB Analyze
    let fb_moves = &phase_moves[0].1;
    for m in fb_moves {
        let base = m.trim_end_matches(|c| c == '\'' || c == '2');
        match base {
            "R" => { penalties += 10; issues.push(format!("[HIGH] FB: {} - R moves shouldn't be in FB", m)); }
            "S" => { penalties += 10; issues.push(format!("[HIGH] FB: {} - S slice is unusual in FB", m)); }
            "E" => { penalties += 5; issues.push(format!("[MEDIUM] FB: {} - E slice is unusual", m)); }
            _ => {}
        }
    }
    if fb_moves.len() > 12 {
        penalties += (fb_moves.len() - 12) as i32;
        issues.push(format!("[INFO] FB: {} moves (target ≤9)", fb_moves.len()));
    }

    // SB Analyze
    let sb_moves = &phase_moves[1].1;
    let (mut m_count, mut r_count) = (0, 0);
    for m in sb_moves {
        let base = m.trim_end_matches(|c| c == '\'' || c == '2');
        match base {
            "L" | "l" => { penalties += 20; issues.push(format!("[CRITICAL] SB: {} - L moves DESTROY FB!", m)); }
            "F" | "B" => { penalties += 10; issues.push(format!("[HIGH] SB: {} - {} move breaks FB", m, base)); }
            "S" => { penalties += 10; issues.push(format!("[HIGH] SB: {} - S slice shouldn't be in SB", m)); }
            "M" => m_count += 1,
            "r" => r_count += 1,
            _ => {}
        }
    }
    let roux_ratio = if sb_moves.is_empty() { 0.0 } else { (m_count + r_count) as f32 / sb_moves.len() as f32 };
    if roux_ratio < 0.2 && sb_moves.len() > 5 {
        issues.push(format!("[INFO] SB: Low M/r usage ({:.0}%)", roux_ratio * 100.0));
    }
    if sb_moves.len() > 16 {
        penalties += (sb_moves.len() - 16) as i32;
        issues.push(format!("[INFO] SB: {} moves (target ≤12)", sb_moves.len()));
    }

    // LSE Analyze
    for idx in 3..=5 {
        for m in &phase_moves[idx].1 {
            let base = m.trim_end_matches(|c| c == '\'' || c == '2');
            match base {
                "M" | "U" => {}
                "R" | "L" | "F" | "B" | "D" | "r" | "l" => {
                    penalties += 20;
                    issues.push(format!("[CRITICAL] {}: {} - Only M/U allowed", phase_moves[idx].0, m));
                }
                "x" | "y" | "z" => {
                    penalties += 10;
                    issues.push(format!("[HIGH] {}: {} - No rotations in LSE", phase_moves[idx].0, m));
                }
                _ => {}
            }
        }
    }

    let score = (100 - penalties).max(0);
    let class = match score {
        90..=100 => "Elite Roux ✨ ",
        75..=89 => "Good Roux 👍 ",
        60..=74 => "Developing Roux 📈 ",
        40..=59 => "Hybrid Style ⚠️ ",
        _ => "CFOP-style 🔄 ",
    };
    
    println!("Roux Purity Score: {}/100 ({} )", score, class);
    if !issues.is_empty() {
        println!("\nIssues Detected:");
        for issue in &issues { println!("  {}", issue); }
    } else {
        println!("\nNo style issues detected! Clean Roux solve.");
    }
    println!("===============================\n");
}
