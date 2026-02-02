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
    
    // Parse scramble from CLI arguments
    let args: Vec<String> = std::env::args().collect();
    let scramble_moves: Vec<String> = if args.len() > 1 {
        // Assume first argument is the full scramble string (quoted)
        args[1].split_whitespace().map(|s| s.to_string()).collect()
    } else {
        Vec::new()
    };

    // Timing and Scramble Progress
    let mut last_move = Instant::now();
    let mut scramble_index = 0;
    use rand::seq::SliceRandom;
    let move_options = ["U", "U'", "U2", "D", "D'", "D2", "L", "L'", "L2", "R", "R'", "R2", "F", "F'", "F2", "B", "B'", "B2"];

    println!("Window created - starting render loop");
    if !scramble_moves.is_empty() {
        println!("Playing Scramble: {:?}", scramble_moves);
    } else {
        println!("No scramble provided. Falling back to Random moves every 10s.");
        println!("Auto-gyro disabled. Manual orbit controls active.");
    }

    // Main render loop
    window.render_loop(move |mut frame_input| {
        // Update viewport and handle input
        render_state.set_viewport(frame_input.viewport);
        render_state.handle_events(&mut frame_input.events);
        
        // Handle Move Playback
        if !scramble_moves.is_empty() && scramble_index < scramble_moves.len() {
            if last_move.elapsed() >= Duration::from_millis(500) {
                let move_str = &scramble_moves[scramble_index];
                println!("[Scramble {}/{}] Applying: {}", scramble_index + 1, scramble_moves.len(), move_str);
                cube_state.apply_move(move_str);
                //cube_state.dump_debug();
                scramble_index += 1;
                last_move = Instant::now();
            }
        } else if scramble_moves.is_empty() {
            // Random mode if no scramble provided
            if last_move.elapsed() >= Duration::from_secs(10) {
                let mut rng = rand::thread_rng();
                if let Some(&move_str) = move_options.choose(&mut rng) {
                    println!("[10s] Applying Random Move: {}", move_str);
                    cube_state.apply_move(move_str);
                    //cube_state.dump_debug();
                }
                last_move = Instant::now();
            }
        }

        // Render frame (render state handles everything)
        render_state.update_cube_state(&cube_state.get_facelets(), None);
        render_state.render_frame(&frame_input.screen());
        
        FrameOutput::default()
    });
}
