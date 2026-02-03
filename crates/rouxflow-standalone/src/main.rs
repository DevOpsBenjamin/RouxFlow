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
    let scramble_moves: Vec<String> = if args.len() > 1 {
        args[1].split_whitespace().map(|s| s.to_string()).collect()
    } else {
        Vec::new()
    };
    
    let solve_moves: Vec<String> = if args.len() > 2 {
        args[2].split_whitespace().map(|s| s.to_string()).collect()
    } else {
        Vec::new()
    };

    // Timing and Progress
    let mut last_move = Instant::now();
    let mut move_idx = 0;
    let mut is_solving = false;
    
    use rand::seq::SliceRandom;
    let move_options = ["U", "U'", "U2", "D", "D'", "D2", "L", "L'", "L2", "R", "R'", "R2", "F", "F'", "F2", "B", "B'", "B2"];

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
        
        // 1. Play Scramble (Fast: 200ms)
        if !is_solving && move_idx < scramble_moves.len() {
            if last_move.elapsed() >= Duration::from_millis(100) {
                cube_state.apply_move(&scramble_moves[move_idx]);
                move_idx += 1;
                last_move = now;
                if move_idx == scramble_moves.len() {
                    println!("Scramble complete. Starting solve playback in 1s...");
                    is_solving = true;
                    move_idx = 0;
                    last_move = now + Duration::from_secs(1); // Small delay before solve
                }
            }
        } 
        // 2. Play Solve (Slower: 800ms)
        else if is_solving && move_idx < solve_moves.len() {
            if last_move.elapsed() >= Duration::from_millis(800) {
                let m = &solve_moves[move_idx];
                cube_state.apply_move(m);
                move_idx += 1;
                last_move = now;
                
                let fb = if cube_state.is_fb_solved() { "✅" } else { "❌" };
                let sb = if cube_state.is_sb_solved() { "✅" } else { "❌" };
                let cmll = if cube_state.is_cmll_solved() { "✅" } else { "❌" };
                let bad_edges = cube_state.count_bad_edges();
                println!("[Solve {}/{}] {:<3} | FB: {} | SB: {} | CMLL: {} | EO: {} bad", 
                    move_idx, solve_moves.len(), m, fb, sb, cmll, bad_edges);
            }
        }
        // 3. Random fallback
        else if scramble_moves.is_empty() {
            if last_move.elapsed() >= Duration::from_secs(10) {
                let mut rng = rand::thread_rng();
                if let Some(&m) = move_options.choose(&mut rng) {
                    println!("[Random] Applying: {}", m);
                    cube_state.apply_move(m);
                }
                last_move = now;
            }
        }

        // Render frame
        render_state.update_cube_state(&cube_state.get_facelets(), None);
        render_state.render_frame(&frame_input.screen());
        
        FrameOutput::default()
    });
}
