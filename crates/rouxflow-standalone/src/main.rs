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
    
    // Timing
    let mut last_update = Instant::now(); // Combined gyro + state update
    let mut last_move = Instant::now();
    let mut rotation_count = 0;
    
    println!("Window created - starting render loop");
    println!("Every 1s: update cube state + gyro rotation");
    println!("Every 10s: apply U move");

    // Main render loop
    window.render_loop(move |mut frame_input| {
        // Update viewport and handle input
        render_state.set_viewport(frame_input.viewport);
        render_state.handle_events(&mut frame_input.events);
        
        // Every 1s: update cube state + simulate gyro
        if last_update.elapsed() >= Duration::from_secs(1) {
            rotation_count += 1;
            
            // Rotate on X and Y in opposite directions (PI/16 and -PI/16)
            let angle = (std::f32::consts::PI / 16.0) * rotation_count as f32;
            
            let axis_x = cgmath::Vector3::new(1.0, 0.0, 0.0);
            let axis_y = cgmath::Vector3::new(0.0, 1.0, 0.0);
            
            let rotation_x = cgmath::Quaternion::from_axis_angle(axis_x, cgmath::Rad(angle));
            let rotation_y = cgmath::Quaternion::from_axis_angle(axis_y, cgmath::Rad(-angle)); // Opposite direction
            
            // Combine rotations
            let rotation = rotation_x * rotation_y;
            
            // Update cube_state orientation
            cube_state.orientation = Some(rouxflow_core::cube::Quaternion {
                x: rotation.v.x,
                y: rotation.v.y,
                z: rotation.v.z,
                w: rotation.s,
            });
            
            // Send updated state to render
            let orientation = cube_state.orientation.map(|q| (q.x, q.y, q.z, q.w));
            render_state.update_cube_state(&cube_state.stickers(), orientation);
            
            last_update = Instant::now();
        }
        
        // Every 10s: apply U move
        if last_move.elapsed() >= Duration::from_secs(10) {
            println!("[10s] Applying U move");
            cube_state.apply_move("U");
            cube_state.dump_debug();
            last_move = Instant::now();
        }
        
        // Render frame (render state handles everything)
        let orientation = cube_state.orientation.map(|q| (q.x, q.y, q.z, q.w));
        render_state.update_cube_state(&cube_state.get_facelets(), orientation);
        render_state.render_frame(&frame_input.screen());
        
        FrameOutput::default()
    });
}
