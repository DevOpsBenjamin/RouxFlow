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

#[wasm_bindgen]
pub struct RouxRenderer {
    state: Rc<RefCell<SharedState>>,
}

#[wasm_bindgen]
impl RouxRenderer {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas_id: String) -> Result<RouxRenderer, JsValue> {
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
            vec3(2.5, 2.5, 5.0), // Closer/more front-facing view
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
                    albedo: Srgba::new(255, 80, 80, 255), // Brighter red
                    ..Default::default()
                },
            ),
        );

        // cgmath quaternion: w, x, y, z
        let identity = Quaternion::new(1.0, 0.0, 0.0, 0.0);

        let state = Rc::new(RefCell::new(SharedState {
            raw_rotation: identity,
            rotation_offset: identity,
            display_rotation: identity,
            has_gyro: false,
        }));

        let loop_state = state.clone();

        window.render_loop(move |mut frame_input| {
            camera.set_viewport(frame_input.viewport);
            control.handle_events(&mut camera, &mut frame_input.events);

            let current_display_rot;
            {
                let s = loop_state.borrow();
                current_display_rot = s.display_rotation;
            }

            // Check identity (w approx 1)
            let is_identity = current_display_rot.s > 0.999; 
            // Note: cgmath Quaternion uses .s for scalar (w) and .v for vector part (x,y,z)

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

        log::info!("RouxRenderer started loop on canvas: {}", canvas_id);

        Ok(RouxRenderer { state })
    }

    pub fn set_gyro_enabled(&mut self, enabled: bool) {
        let mut s = self.state.borrow_mut();
        s.has_gyro = enabled;
        if !enabled {
            s.display_rotation = Quaternion::new(1.0, 0.0, 0.0, 0.0);
        }
    }

    pub fn update_rotation(&mut self, x: f32, y: f32, z: f32, w: f32) {
        let mut s = self.state.borrow_mut();
        // cgmath: new(w, x, y, z)
        let raw = Quaternion::new(w, x, y, z); 
        s.raw_rotation = raw;
        s.display_rotation = s.rotation_offset * raw;
    }

    pub fn reset_gyro(&mut self) {
        let mut s = self.state.borrow_mut();
        s.rotation_offset = s.raw_rotation.invert();
        s.display_rotation = Quaternion::new(1.0, 0.0, 0.0, 0.0);
    }
}
