use wasm_bindgen::prelude::*;
use three_d::*;
use std::rc::Rc;
use std::cell::RefCell;
use wasm_bindgen::JsCast;

// CGMath types used by three-d
struct SharedState {
    raw_rotation: Quaternion<f32>,
    rotation_offset: Quaternion<f32>,
    display_rotation: Quaternion<f32>,
    has_gyro: bool,
}

// Thread-local global state to allow access across the WASM boundary
thread_local! {
    static STATE: Rc<RefCell<Option<SharedState>>> = Rc::new(RefCell::new(None));
}

// Helper to setup requestAnimationFrame
fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    web_sys::window()
        .unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register requestAnimationFrame");
}

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

    // Base mesh: Centered at (0,0,0)
    let mut cpu_mesh = CpuMesh::cube();
    cpu_mesh.transform(&Mat4::from_translation(vec3(-0.5, -0.5, -0.5))).unwrap();

    // Grille 5x5 de 25 cubes, centrée sur (0,0,0)
    let mut models: Vec<(Gm<Mesh, ColorMaterial>, Vector3<f32>)> = Vec::new();
    
    let spacing = 2.5; // Espacement entre les cubes
    
    // Grille 5x5 sur le plan XY (Z=0)
    for x in -2..=2 {
        for y in -2..=2 {
            let pos = vec3(x as f32 * spacing, y as f32 * spacing, 0.0);
            
            // Couleur basée sur les coordonnées
            let color = if x == 0 && y == 0 {
                Srgba::new(255, 255, 255, 255) // Centre = Blanc
            } else {
                let r = ((x + 2) as f32 / 4.0 * 200.0 + 55.0) as u8;
                let g = ((y + 2) as f32 / 4.0 * 200.0 + 55.0) as u8;
                let b = 80u8;
                Srgba::new(r, g, b, 255)
            };
            
            let m = Gm::new(
                Mesh::new(&context, &cpu_mesh),
                ColorMaterial::new(
                    &context,
                    &CpuMaterial {
                        albedo: color,
                        ..Default::default()
                    },
                ),
            );
            
            models.push((m, pos));
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
        // Read canvas dimensions
        let rect = canvas_for_loop.get_bounding_client_rect();
        let dpr = web_sys::window().unwrap().device_pixel_ratio();
        let canvas_width = (rect.width() * dpr) as u32;
        let canvas_height = (rect.height() * dpr) as u32;
        
        // Sync canvas buffer size to CSS size
        if canvas_for_loop.width() != canvas_width || canvas_for_loop.height() != canvas_height {
            canvas_for_loop.set_width(canvas_width);
            canvas_for_loop.set_height(canvas_height);
        }
        
        // Update camera viewport
        let viewport = Viewport {
            x: 0,
            y: 0,
            width: canvas_width,
            height: canvas_height,
        };
        camera.set_viewport(viewport);
        
        // Get current rotation
        let current_display_rot = {
            let borrowed_opt = loop_state_rc.borrow();
            if let Some(s) = borrowed_opt.as_ref() {
                s.display_rotation
            } else {
                Quaternion::new(1.0, 0.0, 0.0, 0.0)
            }
        };
        
        let is_identity = current_display_rot.s > 0.999;
        let rotation_mat = if is_identity {
            Mat4::identity()
        } else {
            Mat4::from(current_display_rot)
        };
        
        // Create render target for the screen
        let render_target = RenderTarget::screen(&context_for_loop, canvas_width, canvas_height);
        
        // Clear
        render_target.clear(ClearState::color_and_depth(0.05, 0.05, 0.08, 1.0, 1.0));
        
        // Render all models
        let scale_mat = Mat4::from_scale(0.7);
        
        for (model, pos) in &mut models {
            let translation_mat = Mat4::from_translation(*pos);
            let transform = rotation_mat * translation_mat * scale_mat;
            model.set_transformation(transform);
            
            render_target.render(&camera, &[model], &[]);
        }
        
        // Continue loop
        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));
    
    // Start the loop
    request_animation_frame(g.borrow().as_ref().unwrap());
    
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
