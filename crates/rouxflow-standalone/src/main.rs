use three_d::*;
use rouxflow_core::cube::CubeState;
use rouxflow_render::RenderState;
use std::time::{Duration, Instant};

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
        eprintln!("Usage: rouxflow-standalone \"<scramble>\" \"<solve>\"");
        eprintln!("Example: rouxflow-standalone \"R U R' U'\" \"x2 y U R U' R'\"");
        std::process::exit(1);
    }
    
    let scramble_moves: Vec<String> = args[1].split_whitespace().map(|s| s.to_string()).collect();
    let solve_moves: Vec<String> = args[2].split_whitespace().map(|s| s.to_string()).collect();

    // Timing and Progress
    let mut last_move = Instant::now();
    let mut move_idx = 0;
    let mut is_solving = false;
    
    // Roux Phase Tracking
    #[derive(PartialEq, Clone, Copy, Debug)]
    enum RouxPhase { FB, SB, CMLL, LSE_EO, LSE_ULUR, LSE_L4E, DONE }
    let mut current_phase = RouxPhase::FB;
    let mut phase_moves: Vec<(String, Vec<String>)> = vec![
        ("FB".to_string(), Vec::new()),
        ("SB".to_string(), Vec::new()),
        ("CMLL".to_string(), Vec::new()),
        ("LSE (EO)".to_string(), Vec::new()),
        ("LSE (ULUR)".to_string(), Vec::new()),
        ("LSE (Finish)".to_string(), Vec::new()),
    ];
    


    println!("Window created - starting render loop");
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
        // 2. Play Solve (500ms between moves, 400ms animation)
        else if is_solving && move_idx < solve_moves.len() {
            if last_move.elapsed() >= Duration::from_millis(500) {
                let m = &solve_moves[move_idx];
                cube_state.apply_move(m);
                render_state.trigger_move_anim(m, 0.4);

                move_idx += 1;
                last_move = now;

                // Add move to current phase repository
                let phase_idx = match current_phase {
                    RouxPhase::FB => 0,
                    RouxPhase::SB => 1,
                    RouxPhase::CMLL => 2,
                    RouxPhase::LSE_EO => 3,
                    RouxPhase::LSE_ULUR => 4,
                    _ => 5,
                };
                phase_moves[phase_idx].1.push(m.clone());

                // Update Phase State Machine (triggers only once per phase)
                let fb = if cube_state.is_fb_solved() { "✅" } else { "❌" };
                let sb = if cube_state.is_sb_solved() { "✅" } else { "❌" };
                let cmll = if cube_state.is_cmll_solved() { "✅" } else { "❌" };
                let bad_edges = cube_state.count_bad_edges();
                let ulur = if cube_state.is_ul_ur_placed() { "✅" } else { "❌" };
                let l4e = if cube_state.is_l4e_solved() { "✅" } else { "❌" };
                
                println!("[Solve {:2}/{:2}] {:<3} | FB: {} | SB: {} | CMLL: {} | EO: {} bad | ULUR: {} | L4E: {}", 
                    move_idx, solve_moves.len(), m, fb, sb, cmll, bad_edges, ulur, l4e);

                // Phase transition logic
                if current_phase == RouxPhase::FB && cube_state.is_fb_solved() {
                    current_phase = RouxPhase::SB;
                    println!(">>> Progress: First Block Finished");
                } else if current_phase == RouxPhase::SB && cube_state.is_sb_solved() {
                    current_phase = RouxPhase::CMLL;
                    println!(">>> Progress: Second Block Finished");
                } else if current_phase == RouxPhase::CMLL && cube_state.is_cmll_solved() {
                    current_phase = RouxPhase::LSE_EO;
                    println!(">>> Progress: CMLL Finished");
                } else if current_phase == RouxPhase::LSE_EO && bad_edges == 0 {
                    current_phase = RouxPhase::LSE_ULUR;
                    println!(">>> Progress: LSE EO Finished");
                } else if current_phase == RouxPhase::LSE_ULUR && cube_state.is_ul_ur_placed() {
                    current_phase = RouxPhase::LSE_L4E;
                    println!(">>> Progress: LSE UL/UR Finished");
                } else if current_phase == RouxPhase::LSE_L4E && cube_state.is_l4e_solved() {
                    current_phase = RouxPhase::DONE;
                    println!(">>> Progress: Solve Finished");
                }

                if move_idx == solve_moves.len() {
                    println!("\n===== ROUX SOLVE ANALYSIS =====");
                    let mut total_moves = 0;
                    for (name, moves) in &phase_moves {
                        if !moves.is_empty() {
                            println!("{:<15}: {:2} moves -> {}", name, moves.len(), moves.join(" "));
                            total_moves += moves.len();
                        }
                    }
                    println!("-------------------------------");
                    println!("Total Moves: {}", total_moves);
                    println!("===============================\n");
                }
            }
        }


        // Render frame
        render_state.update_cube_state(&cube_state.get_facelets(), None);
        render_state.render_frame(&frame_input.screen(), frame_input.elapsed_time as f32 / 1000.0);
        
        FrameOutput::default()
    });
}
