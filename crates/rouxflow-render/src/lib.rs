use wasm_bindgen::prelude::*;
use three_d::*;
use std::rc::Rc;
use std::cell::RefCell;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

// CGMath types used by three-d
struct SharedState {
    raw_rotation: Quaternion<f32>,
    rotation_offset: Quaternion<f32>,
    display_rotation: Quaternion<f32>,
    has_gyro: bool,
}

// Thread-local global state to allow access across the WASM boundary
#[cfg(target_arch = "wasm32")]
thread_local! {
    static STATE: Rc<RefCell<Option<SharedState>>> = Rc::new(RefCell::new(None));
}

// Helper to setup requestAnimationFrame
#[cfg(target_arch = "wasm32")]
fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    web_sys::window()
        .unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register requestAnimationFrame");
}

// Helper to create debug grid models
fn setup_models(context: &Context) -> Vec<(Gm<Mesh, ColorMaterial>, Vector3<f32>)> {
    // Base mesh: Centered at (0,0,0)
    let mut cpu_mesh = CpuMesh::cube();
    cpu_mesh.transform(&Mat4::from_translation(vec3(-0.5, -0.5, -0.5))).unwrap();

    let mut models = Vec::new();
    let spacing = 2.5;
    
    // Grille 5x5 sur le plan XY (Z=0)
    for x in -2..=2 {
        for y in -2..=2 {
            let pos = vec3(x as f32 * spacing, y as f32 * spacing, 0.0);
            
            let color = if x == 0 && y == 0 {
                Srgba::new(255, 255, 255, 255)
            } else {
                let r = ((x + 2) as f32 / 4.0 * 200.0 + 55.0) as u8;
                let g = ((y + 2) as f32 / 4.0 * 200.0 + 55.0) as u8;
                let b = 80u8;
                Srgba::new(r, g, b, 255)
            };
            
            let m = Gm::new(
                Mesh::new(context, &cpu_mesh),
                ColorMaterial::new(
                    context,
                    &CpuMaterial {
                        albedo: color,
                        ..Default::default()
                    },
                ),
            );
            
            models.push((m, pos));
        }
    }
    
    models
}

// ========== NATIVE API (for standalone) ==========
#[cfg(not(target_arch = "wasm32"))]
pub struct RenderState {
    models: Vec<(Gm<Mesh, ColorMaterial>, Vector3<f32>)>,
    display_rotation: cgmath::Quaternion<f32>,
    camera: Camera,
    control: OrbitControl,
}

#[cfg(not(target_arch = "wasm32"))]
impl RenderState {
    /// Initialize render state from GL context
    pub fn new(context: &Context, viewport: Viewport) -> Self {
        let models = setup_models(context);
        let identity = cgmath::Quaternion::new(1.0, 0.0, 0.0, 0.0);
        
        let camera = Camera::new_perspective(
            viewport,
            vec3(0.0, 0.0, 20.0),
            vec3(0.0, 0.0, 0.0),
            vec3(0.0, 1.0, 0.0),
            degrees(45.0),
            0.1,
            100.0,
        );
        
        let control = OrbitControl::new(camera.target().clone(), 0.1, 100.0);
        
        RenderState {
            models,
            display_rotation: identity,
            camera,
            control,
        }
    }
    
    /// Update viewport (when window resizes)
    pub fn set_viewport(&mut self, viewport: Viewport) {
        self.camera.set_viewport(viewport);
    }
    
    /// Handle input events (mouse, keyboard)
    pub fn handle_events(&mut self, events: &mut [Event]) {
        self.control.handle_events(&mut self.camera, events);
    }
    
    /// Update cube state (stickers + orientation)
    pub fn update_cube_state(&mut self, _stickers: &[u8], orientation: Option<(f32, f32, f32, f32)>) {
        if let Some((x, y, z, w)) = orientation {
            self.display_rotation = cgmath::Quaternion::new(w, x, y, z);
        }
        // TODO: Update cube colors from stickers
    }
    
    /// Render one frame
    pub fn render_frame(&mut self, screen: &RenderTarget) {
        let rotation_mat = Mat4::from(self.display_rotation);
        
        screen.clear(ClearState::color_and_depth(0.05, 0.05, 0.08, 1.0, 1.0));
        
        let scale_mat = Mat4::from_scale(0.7);
        
        for (model, pos) in &mut self.models {
            let translation_mat = Mat4::from_translation(*pos);
            let transform = rotation_mat * translation_mat * scale_mat;
            model.set_transformation(transform);
            
            screen.render(&self.camera, &[model], &[]);
        }
    }
}

// ========== WASM API (for web) ==========
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn init_renderer(canvas_id: String) -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    
    // Get canvas element
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas_element = document
        .get_element_by_id(&canvas_id)
        .ok_or("Canvas not found")?
        .dyn_into::<web_sys::HtmlCanvasElement>()?;
    
    // Create WebGL2 context manually
    let webgl_context = canvas_element
        .get_context("webgl2")?
        .ok_or("Failed to get webgl2 context")?
        .dyn_into::<web_sys::WebGl2RenderingContext>()?;
    
    // Wrap in glow Context (three-d's low-level GL abstraction via context module)
    let glow_context = three_d::context::Context::from_webgl2_context(webgl_context);
    
    // Wrap in three-d Context
    let context = Context::from_gl_context(std::sync::Arc::new(glow_context))
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    // Setup models using shared helper
    let mut models = setup_models(&context);

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

    // Setup autonomous render loop
    let loop_state_rc = STATE.with(|s| s.clone());
    
    // Create camera (will be updated each frame with correct viewport)
    let mut camera = Camera::new_perspective(
        Viewport::new_at_origo(1, 1), // Placeholder, updated each frame
        vec3(0.0, 0.0, 20.0),
        vec3(0.0, 0.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        degrees(45.0),
        0.1,
        100.0,
    );
    
    let mut control = OrbitControl::new(camera.target().clone(), 0.1, 100.0);
    
    // Render loop closure
    let f = Rc::new(RefCell::new(None::<Closure<dyn FnMut()>>));
    let g = f.clone();
    
    let canvas_for_loop = canvas_element.clone();
    let context_for_loop = context.clone();
    
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        // Read canvas dimensions dynamically
        let rect = canvas_for_loop.get_bounding_client_rect();
        let dpr = web_sys::window().unwrap().device_pixel_ratio();
        let canvas_width = (rect.width() * dpr) as u32;
        let canvas_height = (rect.height() * dpr) as u32;
        
        // Update camera viewport
        let viewport = Viewport {
            x: 0,
            y: 0,
            width: canvas_width,
            height: canvas_height,
        };
        camera.set_viewport(viewport);
        
        // Read rotation from state
        let display_rotation = loop_state_rc.borrow().as_ref()
            .map(|s| s.display_rotation)
            .unwrap_or(Quaternion::new(1.0, 0.0, 0.0, 0.0));
        
        let rotation_mat = Mat4::from(display_rotation);
        
        // Render target
        let target = RenderTarget::screen(&context_for_loop, canvas_width, canvas_height);
        target.clear(ClearState::color_and_depth(0.05, 0.05, 0.08, 1.0, 1.0));
        
        // Render all models
        let scale_mat = Mat4::from_scale(0.7);
        
        for (model, pos) in &mut models {
            let translation_mat = Mat4::from_translation(*pos);
            let transform = rotation_mat * translation_mat * scale_mat;
            model.set_transformation(transform);
            
            target.render(&camera, &[model], &[]);
        }
        
        // Continue loop
        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));
    
    // Start the loop
    request_animation_frame(g.borrow().as_ref().unwrap());
    
    Ok(())
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn set_gyro_enabled(enabled: bool) {
    STATE.with(|s| {
        if let Some(state) = s.borrow_mut().as_mut() {
            state.has_gyro = enabled;
            if !enabled {
                // Reset orientation when gyro is disabled
                state.raw_rotation = Quaternion::new(1.0, 0.0, 0.0, 0.0);
                state.rotation_offset = Quaternion::new(1.0, 0.0, 0.0, 0.0);
                state.display_rotation = Quaternion::new(1.0, 0.0, 0.0, 0.0);
            }
        }
    });
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn update_gyro(x: f32, y: f32, z: f32, w: f32) {
    STATE.with(|s| {
        if let Some(state) = s.borrow_mut().as_mut() {
            state.raw_rotation = Quaternion::new(w, x, y, z);
            state.display_rotation = state.rotation_offset * state.raw_rotation;
        }
    });
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn reset_gyro() {
    STATE.with(|s| {
        if let Some(state) = s.borrow_mut().as_mut() {
            // Set offset so that current rotation becomes identity
            state.rotation_offset = state.raw_rotation.conjugate();
            state.display_rotation = Quaternion::new(1.0, 0.0, 0.0, 0.0);
        }
    });
}
