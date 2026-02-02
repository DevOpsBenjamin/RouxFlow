use three_d::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use std::rc::Rc;
#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

// CGMath types used by three-d
#[cfg(target_arch = "wasm32")]
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

// Helper to create 3x3x3 Rubik's cube models
fn setup_models(context: &Context) -> Vec<(Gm<Mesh, ColorMaterial>, Vector3<f32>)> {
    // Logic: In three-d 0.17, CpuMesh::cube() is already centered and size 2.0 (from -1.0 to 1.0).
    // - Spacing = 2.0 means centers are at -2, 0, 2.
    // - cubie_size = 0.95 means actual width is 2.0 * 0.95 = 1.9.
    // - Resulting gap = spacing (2.0) - width (1.9) = 0.1.
    let cubie_size = 0.95;   
    let spacing = 2.0;      
    
    // Create base cube mesh (Size 2, centered at origin)
    let mut cpu_mesh = CpuMesh::cube();
    
    // Just scale it down slightly to create the gap
    cpu_mesh.transform(&Mat4::from_scale(cubie_size)).unwrap();

    let mut models = Vec::new();
    
    // Black color for the cubie bodies
    let _cubie_body_color = Srgba::new(20, 20, 20, 255);
    
    // Sticker properties
    let sticker_scale = 0.8; // Sticker takes 80% of cubie face
    let sticker_thickness = 0.02; // Very thin
    let _sticker_offset = 0.96; // Just outside the 0.95 cubie surface
    
    // Create 27 cubies in 3x3x3 grid
    for x in -1..=1 {
        for y in -1..=1 {
            for z in -1..=1 {
                let center_pos = vec3(
                    x as f32 * spacing,
                    y as f32 * spacing,
                    z as f32 * spacing
                );
                
                // 1. ADD CUBIE BODY (Opaque Dark Grey - better for readability)
                let body_color = Srgba::new(45, 45, 45, 255); 

                let mut body = Gm::new(
                    Mesh::new(context, &cpu_mesh),
                    ColorMaterial::new(
                        context,
                        &CpuMaterial {
                            albedo: body_color,
                            roughness: 0.05, // Extra shiny for clear edges
                            metallic: 0.2,
                            ..Default::default()
                        },
                    ),
                );
                // Back to opaque for better readability
                body.material.render_states.blend = Blend::Disabled;
                models.push((body, center_pos));
                
                // 2. ADD STICKERS (Only on external faces)
                
                // Helper to add a sticker: scale is HALF dimensions because base cube is size 2
                let mut add_sticker = |offset: Vector3<f32>, s: Vector3<f32>, color: Srgba| {
                    let mut s_mesh = CpuMesh::cube();
                    // NO translation! It's already centered at origin.
                    s_mesh.transform(&Mat4::from_nonuniform_scale(s.x, s.y, s.z)).unwrap();
                    
                    let s_gm = Gm::new(
                        Mesh::new(context, &s_mesh),
                        ColorMaterial::new(
                            context,
                            &CpuMaterial {
                                albedo: color,
                                roughness: 0.2,
                                ..Default::default()
                            },
                        ),
                    );
                    models.push((s_gm, center_pos + offset));
                };

                // Half-dimensions for scaling the size-2 cube
                let s_half = cubie_size * sticker_scale;
                let t_half = sticker_thickness / 2.0;
                let off = cubie_size + t_half; // Surface is at exactly cubie_size

                // Front (+Z) -> White
                if z == 1 {
                    add_sticker(vec3(0.0, 0.0, off), vec3(s_half, s_half, t_half), Srgba::WHITE);
                }
                // Back (-Z) -> Yellow
                if z == -1 {
                    add_sticker(vec3(0.0, 0.0, -off), vec3(s_half, s_half, t_half), Srgba::new(255, 255, 0, 255));
                }
                // Top (+Y) -> Green
                if y == 1 {
                    add_sticker(vec3(0.0, off, 0.0), vec3(s_half, t_half, s_half), Srgba::GREEN);
                }
                // Bottom (-Y) -> Blue
                if y == -1 {
                    add_sticker(vec3(0.0, -off, 0.0), vec3(s_half, t_half, s_half), Srgba::BLUE);
                }
                // Right (+X) -> Red
                if x == 1 {
                    add_sticker(vec3(off, 0.0, 0.0), vec3(t_half, s_half, s_half), Srgba::RED);
                }
                // Left (-X) -> Orange
                if x == -1 {
                    add_sticker(vec3(-off, 0.0, 0.0), vec3(t_half, s_half, s_half), Srgba::new(255, 165, 0, 255));
                }
            }
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
    ambient_light: AmbientLight,
    directional_light: DirectionalLight,
    directional_light_2: DirectionalLight,
}

#[cfg(not(target_arch = "wasm32"))]
impl RenderState {
    /// Initialize render state from GL context
    pub fn new(context: &Context, viewport: Viewport) -> Self {
        let models = setup_models(context);
        let identity = cgmath::Quaternion::new(1.0, 0.0, 0.0, 0.0);
        
        let camera = Camera::new_perspective(
            viewport,
            vec3(0.0, 0.0, 14.0),  // Camera distance for 3x3x3 cube
            vec3(0.0, 0.0, 0.0),
            vec3(0.0, 1.0, 0.0),
            degrees(45.0),
            0.1,
            100.0,
        );
        
        let control = OrbitControl::new(camera.target().clone(), 0.1, 100.0);
        
        let ambient_light = AmbientLight::new(context, 0.4, Srgba::WHITE);
        let directional_light = DirectionalLight::new(context, 1.0, Srgba::WHITE, &vec3(1.0, -1.0, -1.0));
        let directional_light_2 = DirectionalLight::new(context, 0.6, Srgba::WHITE, &vec3(-1.0, 1.0, 1.0));
        
        RenderState {
            models,
            display_rotation: identity,
            camera,
            control,
            ambient_light,
            directional_light,
            directional_light_2,
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
    /// 
    /// The stickers parameter uses the Twizzle Binary 3x3x3 Format:
    /// https://experiments.cubing.net/cubing.js/spec/binary/
    /// 
    /// Format (~20 bytes):
    /// - EP (Edge Permutation): 29 bits
    /// - EO (Edge Orientation): 11 bits
    /// - CP (Corner Permutation): 17 bits
    /// - CO (Corner Orientation): 13 bits
    /// - MO (Center Orientation): optional
    /// - PO (Puzzle Orientation): optional
    pub fn update_cube_state(&mut self, stickers: &[u8], orientation: Option<(f32, f32, f32, f32)>) {
        if let Some((x, y, z, w)) = orientation {
            self.display_rotation = cgmath::Quaternion::new(w, x, y, z);
        }
        
        // TODO: Decode Twizzle binary format
        // For now, we just accept the binary data without parsing it
        // When we switch from debug grid (5x5) to actual Rubik's cube (3x3x3),
        // we'll need to:
        // 1. Parse EP, EO, CP, CO from binary
        // 2. Map each of 27 cubies to world positions
        // 3. For each cubie, determine which faces are visible
        // 4. Map sticker colors to those visible faces
        
        let _is_solved = stickers.len() >= 2 && stickers[0] == 0x01 && stickers[1] == 0x01;
    }
    
    /// Render one frame
    pub fn render_frame(&mut self, screen: &RenderTarget) {
        let rotation_mat = Mat4::from(self.display_rotation);
        
        // Lighter gray background as requested
        screen.clear(ClearState::color_and_depth(0.25, 0.25, 0.28, 1.0, 1.0));
        
        let lights: &[&dyn Light] = &[&self.ambient_light, &self.directional_light, &self.directional_light_2];
        
        // No arbitrary scale - use real coordinates
        for (model, pos) in &mut self.models {
            let translation_mat = Mat4::from_translation(*pos);
            let transform = rotation_mat * translation_mat;
            model.set_transformation(transform);
            
            screen.render(&self.camera, &[model], lights);
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
