//! 3D cube rendering for RouxFlow using three-d.
//!
//! Shared `RenderState` works on both native and WASM targets.
//! WASM-specific API (init_renderer, etc.) is behind `cfg(target_arch = "wasm32")`.

use three_d::*;

#[cfg(target_arch = "wasm32")]
use std::rc::Rc;
#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

// CGMath types used by three-d
#[cfg(target_arch = "wasm32")]
struct SharedState {
    raw_rotation: Quaternion<f32>,
    rotation_offset: Quaternion<f32>,
    display_rotation: Quaternion<f32>,
    has_gyro: bool,
    facelets: Vec<u8>,
    pending_move: Option<(String, f32)>,
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

                // 1. ADD CUBIE BODIES
                let body_color = Srgba::new(45, 45, 45, 255);
                let mut body = Gm::new(
                    Mesh::new(context, &cpu_mesh),
                    ColorMaterial::new(
                        context,
                        &CpuMaterial {
                            albedo: body_color,
                            roughness: 0.05,
                            metallic: 0.2,
                            ..Default::default()
                        },
                    ),
                );
                body.material.render_states.blend = Blend::Disabled;
                models.push((body, center_pos));
            }
        }
    }

    // 2. ADD STICKERS (Grouped by face: U, R, F, D, L, B)
    let s_half = cubie_size * sticker_scale;
    let t_half = sticker_thickness / 2.0;
    let off = spacing + cubie_size + t_half; // Correct jump to outer surface (approx 2.96)

    let faces = [
        ("U", Srgba::WHITE),
        ("R", Srgba::RED),
        ("F", Srgba::GREEN),
        ("D", Srgba::new(255, 255, 0, 255)),
        ("L", Srgba::new(255, 165, 0, 255)),
        ("B", Srgba::BLUE),
    ];

    for (name, color) in faces {
        for row in -1..=1 {
            for col in -1..=1 {
                let sticker_pos = match name {
                    "U" => vec3(col as f32 * spacing, off, row as f32 * spacing),
                    "D" => vec3(col as f32 * spacing, -off, -row as f32 * spacing),
                    "F" => vec3(col as f32 * spacing, -row as f32 * spacing, off),
                    "B" => vec3(-col as f32 * spacing, -row as f32 * spacing, -off),
                    "R" => vec3(off, -row as f32 * spacing, -col as f32 * spacing),
                    "L" => vec3(-off, -row as f32 * spacing, col as f32 * spacing),
                    _ => unreachable!(),
                };

                let mut s_mesh = CpuMesh::cube();
                s_mesh.transform(&Mat4::from_nonuniform_scale(
                    if name == "R" || name == "L" { t_half } else { s_half },
                    if name == "U" || name == "D" { t_half } else { s_half },
                    if name == "F" || name == "B" { t_half } else { s_half },
                )).unwrap();

                let s_gm = Gm::new(
                    Mesh::new(context, &s_mesh),
                    ColorMaterial::new(context, &CpuMaterial { albedo: color, roughness: 0.2, ..Default::default() }),
                );
                models.push((s_gm, sticker_pos));
            }
        }
    }

    models
}

// Animation types
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum AnimType {
    Global,
    Slice { axis: char, coord: f32 },
}

// ========== SHARED RENDER STATE (Used by both Web and Native) ==========
pub struct RenderState {
    models: Vec<(Gm<Mesh, ColorMaterial>, Vector3<f32>)>,
    pub display_rotation: cgmath::Quaternion<f32>,

    // Animation State
    anim_type: AnimType,
    anim_target_angle: f32, // target angle (e.g. 90, -90, 180)
    anim_progress: f32, // 0.0 to 1.0
    anim_axis: char,
    anim_duration_secs: f32, // how long the animation should take

    // Pending state (applied after animation ends)
    pub(crate) pending_facelets: Option<Vec<u8>>,

    pub camera: Camera,
    pub control: OrbitControl,
    pub ambient_light: AmbientLight,
    directional_light: DirectionalLight,
    directional_light_2: DirectionalLight,
}

impl RenderState {
    /// Initialize render state from GL context
    pub fn new(context: &Context, viewport: Viewport) -> Self {
        let models = setup_models(context);
        let identity = cgmath::Quaternion::new(1.0, 0.0, 0.0, 0.0);

        let camera = Camera::new_perspective(
            viewport,
            vec3(7.0, 6.0, 11.0), // Green front, Red right, White top
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
            anim_type: AnimType::Global,
            anim_target_angle: 0.0,
            anim_axis: 'y',
            anim_progress: 1.0,
            anim_duration_secs: 0.4,
            pending_facelets: None,
            camera,
            control,
            ambient_light,
            directional_light,
            directional_light_2,
        }
    }

    /// Trigger a visual rotation animation for ANY move
    /// duration_secs: how long the animation should take
    pub fn trigger_move_anim(&mut self, move_str: &str, duration_secs: f32) {
        let clean_move = move_str.trim();
        if clean_move.is_empty() { return; }

        let (base, angle) = if clean_move.ends_with("2'") || clean_move.ends_with('2') {
            (&clean_move[0..clean_move.len()-1], 180.0)
        } else if clean_move.ends_with('\'') {
            (&clean_move[0..clean_move.len()-1], -90.0)
        } else {
            (clean_move, 90.0)
        };

        let s = 2.0;

        let (a_type, axis, final_angle) = match base {
            // Standard face moves (outer layer)
            "U" => (AnimType::Slice { axis: 'y', coord: s }, 'y', -angle),
            "D" => (AnimType::Slice { axis: 'y', coord: -s }, 'y', angle),
            "L" => (AnimType::Slice { axis: 'x', coord: -s }, 'x', angle),
            "R" => (AnimType::Slice { axis: 'x', coord: s }, 'x', -angle),
            "F" => (AnimType::Slice { axis: 'z', coord: s }, 'z', -angle),
            "B" => (AnimType::Slice { axis: 'z', coord: -s }, 'z', angle),
            // Middle slice moves
            "M" => (AnimType::Slice { axis: 'x', coord: 0.0 }, 'x', angle),
            "E" => (AnimType::Slice { axis: 'y', coord: 0.0 }, 'y', angle),
            "S" => (AnimType::Slice { axis: 'z', coord: 0.0 }, 'z', -angle),
            // Wide moves (outer + middle, use coord 0.5 as marker)
            "r" => (AnimType::Slice { axis: 'x', coord: 0.5 }, 'x', -angle),
            "l" => (AnimType::Slice { axis: 'x', coord: -0.5 }, 'x', angle),
            "u" => (AnimType::Slice { axis: 'y', coord: 0.5 }, 'y', -angle),
            "d" => (AnimType::Slice { axis: 'y', coord: -0.5 }, 'y', angle),
            "f" => (AnimType::Slice { axis: 'z', coord: 0.5 }, 'z', -angle),
            "b" => (AnimType::Slice { axis: 'z', coord: -0.5 }, 'z', angle),
            // Global rotations
            "x" => (AnimType::Global, 'x', -angle),
            "y" => (AnimType::Global, 'y', -angle),
            "z" => (AnimType::Global, 'z', -angle),
            _ => return,
        };

        self.anim_type = a_type;
        self.anim_axis = axis;
        self.anim_target_angle = final_angle;
        self.anim_progress = 0.0;
        self.anim_duration_secs = duration_secs;
    }

    /// Queue the new state to be applied after animation completes
    pub fn queue_new_state(&mut self, facelets: &[u8]) {
        if facelets.len() == 54 {
            self.pending_facelets = Some(facelets.to_vec());
        }
    }

    /// Apply pending facelets to models (called after animation ends)
    fn apply_pending_state(&mut self) {
        if let Some(facelets) = self.pending_facelets.take() {
            let get_color = |c_idx: u8| match c_idx {
                0 => Srgba::WHITE,
                1 => Srgba::new(255, 255, 0, 255),
                2 => Srgba::GREEN,
                3 => Srgba::BLUE,
                4 => Srgba::RED,
                5 => Srgba::new(255, 165, 0, 255),
                _ => Srgba::BLACK,
            };

            if self.models.len() >= 81 {
                for i in 0..54 {
                    if let Some((gm, _)) = self.models.get_mut(27 + i) {
                        gm.material.color = get_color(facelets[i]);
                    }
                }
            }
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

    /// Legacy method - now just queues state
    pub fn update_cube_state(&mut self, facelets: &[u8], orientation: Option<(f32, f32, f32, f32)>) {
        if let Some((x, y, z, w)) = orientation {
            self.display_rotation = cgmath::Quaternion::new(w, x, y, z);
        }
        self.queue_new_state(facelets);
    }

    /// Render one frame
    pub fn render_frame(&mut self, screen: &RenderTarget, delta_time: f32) {
        use cgmath::Rotation3;

        // Update animation progress
        let was_animating = self.anim_progress < 1.0;
        if self.anim_progress < 1.0 {
            let speed = if self.anim_duration_secs > 0.0 { 1.0 / self.anim_duration_secs } else { 10.0 };
            self.anim_progress = (self.anim_progress + delta_time * speed).min(1.0);
        }

        // When animation JUST finished, OR if we have a pending state and are not animating
        if (was_animating && self.anim_progress >= 1.0) || (self.anim_progress >= 1.0 && self.pending_facelets.is_some()) {
            self.apply_pending_state();
        }

        // Calculate current animation angle (0 -> target during animation)
        let current_angle = if self.anim_progress < 1.0 {
            self.anim_target_angle * self.anim_progress
        } else {
            0.0
        };

        let anim_rot = if current_angle != 0.0 {
            let axis_vec = match self.anim_axis {
                'x' => vec3(1.0, 0.0, 0.0),
                'y' => vec3(0.0, 1.0, 0.0),
                'z' => vec3(0.0, 0.0, 1.0),
                _ => vec3(0.0, 1.0, 0.0),
            };
            cgmath::Quaternion::from_axis_angle(axis_vec, cgmath::Deg(current_angle))
        } else {
            cgmath::Quaternion::new(1.0, 0.0, 0.0, 0.0)
        };

        let global_rot_mat = Mat4::from(self.display_rotation);
        let anim_rot_mat = Mat4::from(anim_rot);

        screen.clear(ClearState::color_and_depth(0.25, 0.25, 0.28, 1.0, 1.0));
        let lights: &[&dyn Light] = &[&self.ambient_light, &self.directional_light, &self.directional_light_2];

        for (model, pos) in &mut self.models {
            let mut final_transform = global_rot_mat;

            // Apply animation if part of slice or global
            let is_in_anim = match self.anim_type {
                AnimType::Global => true,
                AnimType::Slice { axis, coord } => {
                    let p = match axis {
                        'x' => pos.x,
                        'y' => pos.y,
                        'z' => pos.z,
                        _ => 0.0,
                    };
                    // Outer layer: coord > 1.0 or coord < -1.0
                    // Middle slice: coord == 0.0
                    // Wide move: coord = 0.5 (right+middle) or -0.5 (left+middle)
                    if coord > 1.0 { p > 1.0 }
                    else if coord < -1.0 { p < -1.0 }
                    else if coord > 0.1 { p > -1.0 }  // Wide right: everything except left layer
                    else if coord < -0.1 { p < 1.0 }  // Wide left: everything except right layer
                    else { p.abs() < 1.0 }            // Middle slice only
                }
            };

            if is_in_anim {
                final_transform = final_transform * anim_rot_mat;
            }

            let translation_mat = Mat4::from_translation(*pos);
            model.set_transformation(final_transform * translation_mat);
            screen.render(&self.camera, &[model], lights);
        }
    }
}

// ========== WASM API (for web) ==========
// These are public functions callable from rouxflow-wasm entry point.

#[cfg(target_arch = "wasm32")]
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

    // Create RenderState
    let mut render_state = RenderState::new(&context, Viewport::new_at_origo(1, 1));

    let identity = Quaternion::new(1.0, 0.0, 0.0, 0.0);

    // Initialize state
    STATE.with(|s| {
        *s.borrow_mut() = Some(SharedState {
            raw_rotation: identity,
            rotation_offset: identity,
            display_rotation: identity,
            has_gyro: false,
            facelets: vec![0; 54], // Start with correct size
            pending_move: None,
        });
    });

    // Setup autonomous render loop
    let loop_state_rc = STATE.with(|s| s.clone());

    // Render loop closure
    let f = Rc::new(RefCell::new(None::<Closure<dyn FnMut()>>));
    let g = f.clone();

    let canvas_for_loop = canvas_element.clone();
    let context_for_loop = context.clone();
    let mut last_time = web_sys::window().unwrap().performance().unwrap().now();

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        let now = web_sys::window().unwrap().performance().unwrap().now();
        let delta_time = ((now - last_time) / 1000.0) as f32;
        last_time = now;

        // Read canvas dimensions dynamically
        let rect = canvas_for_loop.get_bounding_client_rect();
        let dpr = web_sys::window().unwrap().device_pixel_ratio();
        let canvas_width = (rect.width() * dpr) as u32;
        let canvas_height = (rect.height() * dpr) as u32;

        // Update viewport
        let viewport = Viewport {
            x: 0,
            y: 0,
            width: canvas_width,
            height: canvas_height,
        };
        render_state.set_viewport(viewport);

        // Read rotation, facelets, and pending move from state
        let (display_rotation, facelets, pending_move) = {
            let mut guard = loop_state_rc.borrow_mut();
            match guard.as_mut() {
                Some(s) => {
                    let pm = s.pending_move.take();
                    (s.display_rotation, s.facelets.clone(), pm)
                }
                None => (Quaternion::new(1.0, 0.0, 0.0, 0.0), Vec::new(), None)
            }
        };

        // Trigger move animation if a new move arrived
        if let Some((move_str, duration)) = pending_move {
            render_state.trigger_move_anim(&move_str, duration);
        }

        render_state.update_cube_state(&facelets, None);
        render_state.display_rotation = display_rotation;

        // Render target
        let target = RenderTarget::screen(&context_for_loop, canvas_width, canvas_height);
        render_state.render_frame(&target, delta_time);

        // Continue loop
        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    // Start the loop
    request_animation_frame(g.borrow().as_ref().unwrap());

    Ok(())
}

#[cfg(target_arch = "wasm32")]
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

/// Queue a move animation to be picked up by the render loop.
#[cfg(target_arch = "wasm32")]
pub fn queue_move_anim(move_str: String, duration_secs: f32) {
    STATE.with(|s| {
        if let Some(state) = s.borrow_mut().as_mut() {
            state.pending_move = Some((move_str, duration_secs));
        }
    });
}

#[cfg(target_arch = "wasm32")]
pub fn update_render_state(facelets: Vec<u8>, x: f32, y: f32, z: f32, w: f32) {
    STATE.with(|s| {
        if let Some(state) = s.borrow_mut().as_mut() {
            state.facelets = facelets;
            state.raw_rotation = Quaternion::new(w, x, y, z);
            state.display_rotation = state.rotation_offset * state.raw_rotation;
        }
    });
}

#[cfg(target_arch = "wasm32")]
pub fn reset_gyro() {
    STATE.with(|s| {
        if let Some(state) = s.borrow_mut().as_mut() {
            // Set offset so that current rotation becomes identity
            state.rotation_offset = state.raw_rotation.conjugate();
            state.display_rotation = Quaternion::new(1.0, 0.0, 0.0, 0.0);
        }
    });
}

/// Set an explicit gyro offset quaternion (used by auto-calibration).
/// Unlike reset_gyro() which uses the current raw rotation, this takes
/// a pre-computed offset (typically conjugate of the averaged home orientation).
#[cfg(target_arch = "wasm32")]
pub fn set_gyro_offset(x: f32, y: f32, z: f32, w: f32) {
    STATE.with(|s| {
        if let Some(state) = s.borrow_mut().as_mut() {
            state.rotation_offset = Quaternion::new(w, x, y, z);
            state.display_rotation = state.rotation_offset * state.raw_rotation;
        }
    });
}
