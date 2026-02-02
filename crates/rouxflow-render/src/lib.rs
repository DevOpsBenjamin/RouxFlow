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
        vec3(2.5, 2.5, 5.0),
        vec3(0.0, 0.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        degrees(45.0),
        0.1,
        100.0,
    );

    let mut control = OrbitControl::new(camera.target().clone(), 1.0, 100.0);
    
    let mut model = Gm::new(
        Mesh::new(&context, &CpuMesh::cube()),
        ColorMaterial::new(
            &context,
            &CpuMaterial {
                albedo: Srgba::new(255, 80, 80, 255),
                ..Default::default()
            },
        ),
    );

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

        let rotation_mat = if is_identity {
                Mat4::from_angle_y(radians((frame_input.accumulated_time * 0.0005) as f32))
        } else {
                Mat4::from(current_display_rot)
        };

        model.set_transformation(rotation_mat);

        frame_input.screen().clear(ClearState::color_and_depth(0.05, 0.05, 0.08, 1.0, 1.0))
            .render(&camera, &[&model], &[]);

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
