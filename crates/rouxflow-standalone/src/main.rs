use three_d::*;
use rouxflow_core::cube::CubeState;
use rouxflow_render::RenderState;
use std::time::{Duration, Instant};

#[derive(PartialEq, Clone, Copy, Debug)]
enum RouxPhase { Fb, Sb, Cmll, LseEo, LseUlur, LseL4e, Done }

#[derive(PartialEq, Clone, Copy, Debug)]
enum PlaybackState { Scramble, Solve, AnalysisPause, OptimizationUndo, OptimizationPlay, Finished }

#[derive(PartialEq, Clone, Copy, Debug)]
enum MovePhase { User, AI }


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
    let mut playback_state = PlaybackState::Scramble;
    
    // Optimization Tracking
    let mut optimization_solutions: Vec<Vec<String>> = Vec::new();
    let mut current_opt_idx = 0;
    
    // Save scrambled state for AI search
    let mut scrambled_logic = cube_state.logic.clone();
    
    // Roux Phase Tracking
    let mut current_phase = RouxPhase::Fb;
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
        if playback_state == PlaybackState::Scramble && move_idx < scramble_moves.len() {
            if last_move.elapsed() >= Duration::from_millis(200) {
                let m = &scramble_moves[move_idx];
                cube_state.apply_move(m);
                render_state.trigger_move_anim(m, 0.15);

                move_idx += 1;
                last_move = now;
                if move_idx == scramble_moves.len() {
                    println!("Scramble complete. Starting solve playback...");
                    // Save the scrambled logic for AI search later
                    scrambled_logic = cube_state.logic.clone();
                    playback_state = PlaybackState::Solve;
                    move_idx = 0;
                    last_move = now + Duration::from_secs(1);
                }
            }
        } 
        // 2. Play Solve (Snappy: 150ms between moves, 100ms animation)
        else if playback_state == PlaybackState::Solve && move_idx < solve_moves.len() {
            if last_move.elapsed() >= Duration::from_millis(150) {
                let m = &solve_moves[move_idx];
                cube_state.apply_move(m);
                render_state.trigger_move_anim(m, 0.1);

                move_idx += 1;
                last_move = now;

                // Track move for analysis
                track_move_phase(&mut current_phase, &mut phase_moves, &cube_state, m, move_idx, solve_moves.len(), debug_mode);

                if move_idx == solve_moves.len() {
                    // Start the analysis pause AFTER the last move animation has time to be seen
                    playback_state = PlaybackState::AnalysisPause;
                    last_move = now; // Start 1s timer
                }
            }
        }
        // 3. Pause after Analysis (1s for last move + 10s for analysis)
        else if playback_state == PlaybackState::AnalysisPause {
            // Step A: Wait 1s for the last move animation to definitely finish
            if move_idx == solve_moves.len() && last_move.elapsed() >= Duration::from_millis(1000) {
                print_solve_analysis(&cube_state, &phase_moves);
                
                // Print camera orbit state as requested
                println!("Camera State: Position({:?}), Target({:?})", 
                    render_state.camera.position(), 
                    render_state.camera.target()
                );
                
                println!("\n[Search] Starting FB optimizations...");
                
                // 1. Legacy Solver (Core)
                println!("   -> Legacy Solver: Thinking... ");
                let mut search_cube = CubeState::new();
                search_cube.logic = scrambled_logic.clone();
                let (legacy_sols, legacy_time) = search_cube.find_fb_solutions(100);
                println!("Found {} in {:?}", legacy_sols.len(), legacy_time);
                for (i, sol) in legacy_sols.iter().enumerate() {
                    println!("      [Legacy Sol {}] {}", i + 1, sol.join(" "));
                }
                
                // 2. New AI Solver (Bitboard)
                println!("   -> AI Solver (Bitboard): Thinking... ");
                let bit_cube = rouxflow_ai::bitcube::BitCube::from_facelet(&scrambled_logic);
                let (ai_sols, ai_time) = rouxflow_ai::solver::AISolver::find_fb_solutions(&bit_cube, 100);
                println!("Found {} in {:?}", ai_sols.len(), ai_time);
                for (i, sol) in ai_sols.iter().enumerate() {
                    println!("      [AI Sol {}] {}", i + 1, sol.join(" "));
                }

                optimization_solutions = ai_sols; // Use AI solutions for playback
                
                playback_state = PlaybackState::AnalysisPause;
                cube_state.logic = scrambled_logic.clone();
                
                move_idx += 1; // Mark initial report as done
                last_move = now; // Start the 10s review timer
                println!(">>> Reviewing analysis... (10s total pause)");
            }
            // Step B: Split the 10s pause (5s view solve, 5s prepare)
            else if move_idx > solve_moves.len() {
                let elapsed = last_move.elapsed();
                
                // After 5s: Show we are starting the AI phase
                if move_idx == solve_moves.len() + 1 && elapsed >= Duration::from_secs(5) {
                    println!(">>> 5s elapsed: Preparing for AI solutions...");
                    move_idx += 1; 
                }
                
                // After 10s total: Start FIRST optimization
                if elapsed >= Duration::from_secs(10) {
                    if current_opt_idx < optimization_solutions.len() {
                        let sol_len = optimization_solutions[current_opt_idx].len();
                        println!("\n[AI INFO] Starting Solution {}/{} ({} moves)", current_opt_idx + 1, optimization_solutions.len(), sol_len);
                        println!("Moves: {}", optimization_solutions[current_opt_idx].join(" "));
                        
                        // Force reset to EXACT scrambled state before starting AI solution
                        cube_state.logic = scrambled_logic.clone();
                        render_state.update_cube_state(&cube_state.facelets(), None);
                        
                        playback_state = PlaybackState::OptimizationPlay;
                        move_idx = 0;
                        last_move = now;
                    } else {
                        println!("\nNo optimization solutions found in current depth.");
                        playback_state = PlaybackState::Finished;
                    }
                }
            }
        }
        // New State: Optimization Undo (Plays inverse moves quickly)
        else if playback_state == PlaybackState::OptimizationUndo {
            let undo_moves = CubeState::invert_moves(&optimization_solutions[current_opt_idx - 1]);
            if move_idx < undo_moves.len() {
                if last_move.elapsed() >= Duration::from_millis(150) {
                    let m = &undo_moves[move_idx];
                    cube_state.apply_move(m);
                    render_state.trigger_move_anim(m, 0.1);

                    move_idx += 1;
                    last_move = now;
                }
            } else {
                // Undo finished
                println!(">>> Solution Undo finished. Resetting to exact scramble state.");
                cube_state.logic = scrambled_logic.clone();
                render_state.update_cube_state(&cube_state.facelets(), None);
                
                if current_opt_idx < optimization_solutions.len() {
                    println!(">>> Waiting 5s before next solution...");
                    playback_state = PlaybackState::AnalysisPause; // Reuse AnalysisPause or logic below
                    last_move = now + Duration::from_secs(5); // Wait 5s total (10 - 5)
                    move_idx = solve_moves.len() + 2; // Jump to Step B
                } else {
                    println!("\nAll optimization solutions shown. Solve Complete.");
                    playback_state = PlaybackState::Finished;
                }
            }
        }
        // 4. Play Optimization Solution (3s per move)
        else if playback_state == PlaybackState::OptimizationPlay {
            let current_sol = &optimization_solutions[current_opt_idx];
            if move_idx < current_sol.len() {
                if last_move.elapsed() >= Duration::from_secs(3) {
                    let m = &current_sol[move_idx];
                    println!("   -> AI Move {}/{}: {}", move_idx + 1, current_sol.len(), m);
                    cube_state.apply_move(m);
                    render_state.trigger_move_anim(m, 1.0);

                    move_idx += 1;
                    last_move = now;
                }
            } else {
                // Solution finished - WAIT 5s before undoing for visual verification
                if last_move.elapsed() >= Duration::from_secs(5) {
                    current_opt_idx += 1;
                    println!(">>> Solution {} Finished.", current_opt_idx);
                    
                    if current_opt_idx <= optimization_solutions.len() {
                        println!(">>> Starting INVERTED playback (Undoing solution)...");
                        playback_state = PlaybackState::OptimizationUndo;
                        move_idx = 0;
                        last_move = now;
                    } else {
                        println!("\nAll optimization solutions shown. Solve Complete.");
                        playback_state = PlaybackState::Finished;
                    }
                }
            }
        }
        // (OptimizationGap removed in favor of OptimizationUndo)

        // Render frame
        render_state.update_cube_state(&cube_state.facelets(), None);
        render_state.render_frame(&frame_input.screen(), frame_input.elapsed_time as f32 / 1000.0);
        
        // Exit if finished
        FrameOutput {
            exit: playback_state == PlaybackState::Finished,
            ..Default::default()
        }
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
        RouxPhase::Fb => 0,
        RouxPhase::Sb => 1,
        RouxPhase::Cmll => 2,
        RouxPhase::LseEo => 3,
        RouxPhase::LseUlur => 4,
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
    if *current_phase == RouxPhase::Fb && cube.is_fb_solved() {
        *current_phase = RouxPhase::Sb;
        println!(">>> Progress: First Block Finished [Solve {:2}/{:2}]", idx, total);
    } else if *current_phase == RouxPhase::Sb && cube.is_sb_solved() {
        *current_phase = RouxPhase::Cmll;
        println!(">>> Progress: Second Block Finished [Solve {:2}/{:2}]", idx, total);
    } else if *current_phase == RouxPhase::Cmll && cube.is_cmll_solved() {
        *current_phase = RouxPhase::LseEo;
        println!(">>> Progress: CMLL Finished [Solve {:2}/{:2}]", idx, total);
    } else if *current_phase == RouxPhase::LseEo && bad_edges == 0 {
        *current_phase = RouxPhase::LseUlur;
        println!(">>> Progress: LSE EO Finished [Solve {:2}/{:2}]", idx, total);
    } else if *current_phase == RouxPhase::LseUlur && cube.is_ul_ur_placed() {
        *current_phase = RouxPhase::LseL4e;
        println!(">>> Progress: LSE UL/UR Finished [Solve {:2}/{:2}]", idx, total);
    } else if *current_phase == RouxPhase::LseL4e && cube.is_l4e_solved() {
        *current_phase = RouxPhase::Done;
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
