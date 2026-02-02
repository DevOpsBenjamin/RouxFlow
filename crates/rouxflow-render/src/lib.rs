use wasm_bindgen::prelude::*;
use three_d::*;
use std::rc::Rc;
use std::cell::RefCell;

// CGMath types used by three-d
// We rename them locally if needed or use full names
// Quaternion in cgmath is Quaternion<S>

struct SharedState {
    raw_rotation: Quaternion<f32>,
    rotation_offset: Quaternion<f32>,
    display_rotation: Quaternion<f32>,
    has_gyro: bool,
}

// Thread-local global state to allow access across the WASM boundary
// even if the init function "throws" to start the event loop.
thread_local! {
    static STATE: Rc<RefCell<Option<SharedState>>> = Rc::new(RefCell::new(None));
}

#[wasm_bindgen]
pub fn init_renderer(canvas_id: String) -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    
    let window = Window::new(WindowSettings {
        title: "RouxFlow 3D".to_string(),
        canvas: Some(
            web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .get_element_by_id(&canvas_id)
                .unwrap()
                .dyn_into::<web_sys::HtmlCanvasElement>()
                .unwrap()
        ),
        ..Default::default()
    }).map_err(|e| e.to_string())?;

    let context = window.gl();

    let mut camera = Camera::new_perspective(
        window.viewport(),
        vec3(8.0, 8.0, 16.0),
        vec3(4.5, -4.5, 0.0), // Increased compensation for bottom-right viewport skew
        vec3(0.0, 1.0, 0.0),
        degrees(45.0),
        0.1,
        100.0,
    );

    let mut control = OrbitControl::new(camera.target().clone(), 0.1, 100.0);
    
    // Base mesh: Centered at (0,0,0)
    let mut cpu_mesh = CpuMesh::cube();
    cpu_mesh.transform(&Mat4::from_translation(vec3(-0.5, -0.5, -0.5))).unwrap();
    
    // Debug: Extreme Coordinates
    let mut models = Vec::new();

    // Debug: Grid of size 0.5 cubes for coordinate extrapolation
    let mut models: Vec<(Gm<Mesh, ColorMaterial>, Vector3<f32>)> = Vec::new();
    
    // Grid range from -2 to 2 covers the effective area of a 3x3 cube (approx -1.5 to 1.5) plus margin
    for x in -2..=2 {
        for y in -2..=2 {
            for z in -2..=2 {
                let pos = vec3(x as f32, y as f32, z as f32);
                
                let mut m = Gm::new(
                    Mesh::new(&context, &cpu_mesh),
                    ColorMaterial::new(
                        &context,
                        &CpuMaterial {
                            albedo: Srgba::new(
                                ((x + 3) as f32 * 50.0) as u8, // X determines unstable Red
                                ((y + 3) as f32 * 50.0) as u8, // Y determines unstable Green
                                ((z + 3) as f32 * 50.0) as u8, // Z determines unstable Blue
                                255
                            ),
                            ..Default::default()
                        },
                    ),
                );

                // Scale 0.5 then Translate to integer position
                m.set_transformation(
                    Mat4::from_translation(pos) * Mat4::from_scale(0.5)
                );
                
                models.push((m, pos));
            }
        }
    }

    let identity = Quaternion::new(1.0, 0.0, 0.0, 0.0);

    // Initialize state
    STATE.with(|s| {
        *s.borrow_mut() = Some(SharedState {
            raw_rotation: identity,
            rotation_offset: identity,
            display_rotation: identity,
            has_gyro: false,
        });
    });

    // Capture state for the loop
    let loop_state_rc = STATE.with(|s| s.clone());

    window.render_loop(move |mut frame_input| {
        camera.set_viewport(frame_input.viewport);
        control.handle_events(&mut camera, &mut frame_input.events);

        let current_display_rot;
        {
            let borrowed_opt = loop_state_rc.borrow();
            if let Some(s) = borrowed_opt.as_ref() {
                current_display_rot = s.display_rotation;
            } else {
                current_display_rot = Quaternion::new(1.0, 0.0, 0.0, 0.0);
            }
        }

        let is_identity = current_display_rot.s > 0.999; 

        // User requested static calibration: Disable auto-rotation
        let rotation_mat = if is_identity {
             Mat4::identity()
        } else {
             Mat4::from(current_display_rot)
        };

        frame_input.screen().clear(ClearState::color_and_depth(0.05, 0.05, 0.08, 1.0, 1.0));

        let scale_mat = Mat4::from_scale(0.92); // 0.92 scale for nice gaps

        for (model, pos) in &mut models {
            // Transform Order: Scale -> Translate -> Rotate (Local to Global)
            // 1. Scale the cubie (local)
            // 2. Translate to its grid position (local assembly)
            // 3. Rotate the entire assembly (global)
            let translation_mat = Mat4::from_translation(*pos);
            let transform = rotation_mat * translation_mat * scale_mat;
            
            model.set_transformation(transform);
            
            frame_input.screen().render(&camera, &[model], &[]);
        }

        FrameOutput::default()
    });

    Ok(())
}

#[wasm_bindgen]
pub fn set_gyro_enabled(enabled: bool) {
    STATE.with(|s| {
        if let Some(state) = s.borrow_mut().as_mut() {
            state.has_gyro = enabled;
            if !enabled {
                state.display_rotation = Quaternion::new(1.0, 0.0, 0.0, 0.0);
            }
        }
    });
}

#[wasm_bindgen]
pub fn update_rotation(x: f32, y: f32, z: f32, w: f32) {
    STATE.with(|s| {
        if let Some(state) = s.borrow_mut().as_mut() {
            let raw = Quaternion::new(w, x, y, z); 
            state.raw_rotation = raw;
            state.display_rotation = state.rotation_offset * raw;
        }
    });
}

#[wasm_bindgen]
pub fn reset_gyro() {
    STATE.with(|s| {
        if let Some(state) = s.borrow_mut().as_mut() {
            state.rotation_offset = state.raw_rotation.invert();
            state.display_rotation = Quaternion::new(1.0, 0.0, 0.0, 0.0);
        }
    });
}
