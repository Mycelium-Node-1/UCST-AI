//! Native Glow-backed reference shader for SphereWorld Lab.
//!
//! The shader is a visualization artifact. It receives only scalar, derived
//! presentation uniforms; canonical SphereWorld state and CPU patch compilation
//! remain the authority for validation, digests, and generated geometry.

use eframe::egui;
use egui_glow::glow::{self, HasContext};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy)]
pub struct ShaderUniforms {
    pub boundary_latitude_degrees: f32,
    pub seam_skirts_enabled: bool,
    pub split_enabled: bool,
    pub yaw: f32,
    pub pitch: f32,
    pub zoom: f32,
}

struct ShaderState {
    program: glow::Program,
    vao: glow::VertexArray,
    u_boundary_latitude: Option<glow::UniformLocation>,
    u_seam_skirts: Option<glow::UniformLocation>,
    u_split_enabled: Option<glow::UniformLocation>,
    u_yaw: Option<glow::UniformLocation>,
    u_pitch: Option<glow::UniformLocation>,
    u_zoom: Option<glow::UniformLocation>,
    u_aspect: Option<glow::UniformLocation>,
    uniforms: ShaderUniforms,
}

/// A small GPU renderer connected through an egui Glow paint callback.
pub struct SphereWorldShader {
    gl: Arc<glow::Context>,
    state: Arc<Mutex<ShaderState>>,
}

impl SphereWorldShader {
    pub fn new(gl: Arc<glow::Context>) -> Result<Self, String> {
        let state = unsafe { create_state(&gl)? };
        Ok(Self {
            gl,
            state: Arc::new(Mutex::new(state)),
        })
    }

    pub fn update(&self, uniforms: ShaderUniforms) {
        if let Ok(mut state) = self.state.lock() {
            state.uniforms = uniforms;
        }
    }

    pub fn paint(&self, ui: &mut egui::Ui, rect: egui::Rect) {
        let state = Arc::clone(&self.state);
        let callback = egui_glow::CallbackFn::new(move |info, painter| {
            let gl = painter.gl();
            let Ok(state) = state.lock() else {
                return;
            };
            let aspect = if info.viewport.height() > 0.0 {
                info.viewport.width() / info.viewport.height()
            } else {
                1.0
            };
            unsafe {
                gl.use_program(Some(state.program));
                gl.bind_vertex_array(Some(state.vao));
                gl.uniform_1_f32(
                    state.u_boundary_latitude.as_ref(),
                    state.uniforms.boundary_latitude_degrees,
                );
                gl.uniform_1_f32(
                    state.u_seam_skirts.as_ref(),
                    if state.uniforms.seam_skirts_enabled {
                        1.0
                    } else {
                        0.0
                    },
                );
                gl.uniform_1_f32(
                    state.u_split_enabled.as_ref(),
                    if state.uniforms.split_enabled {
                        1.0
                    } else {
                        0.0
                    },
                );
                gl.uniform_1_f32(state.u_yaw.as_ref(), state.uniforms.yaw);
                gl.uniform_1_f32(state.u_pitch.as_ref(), state.uniforms.pitch);
                gl.uniform_1_f32(state.u_zoom.as_ref(), state.uniforms.zoom);
                gl.uniform_1_f32(state.u_aspect.as_ref(), aspect);
                gl.draw_arrays(glow::TRIANGLES, 0, 3);
                gl.bind_vertex_array(None);
                gl.use_program(None);
            }
        });
        ui.painter().add(egui::PaintCallback {
            rect,
            callback: Arc::new(callback),
        });
    }
}

impl Drop for SphereWorldShader {
    fn drop(&mut self) {
        if let Ok(state) = self.state.lock() {
            unsafe {
                self.gl.delete_vertex_array(state.vao);
                self.gl.delete_program(state.program);
            }
        }
    }
}

unsafe fn create_state(gl: &glow::Context) -> Result<ShaderState, String> {
    let vertex = compile_shader(gl, glow::VERTEX_SHADER, VERTEX_SHADER)?;
    let fragment = compile_shader(gl, glow::FRAGMENT_SHADER, FRAGMENT_SHADER)?;
    let program = gl.create_program().map_err(|error| error.to_string())?;
    gl.attach_shader(program, vertex);
    gl.attach_shader(program, fragment);
    gl.link_program(program);
    gl.detach_shader(program, vertex);
    gl.detach_shader(program, fragment);
    gl.delete_shader(vertex);
    gl.delete_shader(fragment);
    if !gl.get_program_link_status(program) {
        let error = gl.get_program_info_log(program);
        gl.delete_program(program);
        return Err(format!("SphereWorld shader link failed: {error}"));
    }
    let vao = gl
        .create_vertex_array()
        .map_err(|error| error.to_string())?;
    Ok(ShaderState {
        program,
        vao,
        u_boundary_latitude: gl.get_uniform_location(program, "u_boundary_latitude"),
        u_seam_skirts: gl.get_uniform_location(program, "u_seam_skirts"),
        u_split_enabled: gl.get_uniform_location(program, "u_split_enabled"),
        u_yaw: gl.get_uniform_location(program, "u_yaw"),
        u_pitch: gl.get_uniform_location(program, "u_pitch"),
        u_zoom: gl.get_uniform_location(program, "u_zoom"),
        u_aspect: gl.get_uniform_location(program, "u_aspect"),
        uniforms: ShaderUniforms {
            boundary_latitude_degrees: 70.0,
            seam_skirts_enabled: true,
            split_enabled: false,
            yaw: 0.0,
            pitch: 0.18,
            zoom: 1.0,
        },
    })
}

unsafe fn compile_shader(
    gl: &glow::Context,
    kind: u32,
    source: &str,
) -> Result<glow::Shader, String> {
    let shader = gl.create_shader(kind).map_err(|error| error.to_string())?;
    gl.shader_source(shader, source);
    gl.compile_shader(shader);
    if gl.get_shader_compile_status(shader) {
        Ok(shader)
    } else {
        let error = gl.get_shader_info_log(shader);
        gl.delete_shader(shader);
        Err(format!("SphereWorld shader compilation failed: {error}"))
    }
}

const VERTEX_SHADER: &str = r#"#version 330
out vec2 v_uv;
void main() {
    vec2 positions[3] = vec2[3](vec2(-1.0, -1.0), vec2(3.0, -1.0), vec2(-1.0, 3.0));
    vec2 position = positions[gl_VertexID];
    v_uv = position * 0.5 + 0.5;
    gl_Position = vec4(position, 0.0, 1.0);
}
"#;

const FRAGMENT_SHADER: &str = r#"#version 330
in vec2 v_uv;
out vec4 out_color;

uniform float u_boundary_latitude;
uniform float u_seam_skirts;
uniform float u_split_enabled;
uniform float u_yaw;
uniform float u_pitch;
uniform float u_zoom;
uniform float u_aspect;

mat3 rotate_y(float angle) {
    float c = cos(angle);
    float s = sin(angle);
    return mat3(c, 0.0, s, 0.0, 1.0, 0.0, -s, 0.0, c);
}

mat3 rotate_x(float angle) {
    float c = cos(angle);
    float s = sin(angle);
    return mat3(1.0, 0.0, 0.0, 0.0, c, -s, 0.0, s, c);
}

void main() {
    vec2 p = (v_uv * 2.0 - 1.0) * vec2(u_aspect, 1.0) / max(u_zoom, 0.1);
    float radius_sq = dot(p, p);
    if (radius_sq > 0.92) {
        discard;
    }
    float z = sqrt(max(0.0, 1.0 - radius_sq));
    vec3 direction = normalize(rotate_x(u_pitch) * rotate_y(u_yaw) * vec3(p, z));
    float latitude = degrees(asin(clamp(direction.y, -1.0, 1.0)));
    float polar_boundary = smoothstep(u_boundary_latitude - 1.5, u_boundary_latitude + 1.5, abs(latitude));
    vec3 terrain = mix(vec3(0.14, 0.68, 0.63), vec3(0.93, 0.29, 0.25), polar_boundary);
    float lighting = 0.42 + 0.58 * max(0.0, dot(direction, normalize(vec3(-0.3, 0.5, 1.0))));
    vec3 color = terrain * lighting;

    // Amber marks the selected Pos Z patch's derived seam-skirt region. It is
    // a GPU visualization of the CPU seam-skirt policy, not new world geometry.
    float pos_z_face = step(max(abs(direction.x), abs(direction.y)), direction.z);
    vec2 local = (direction.xy / max(direction.z, 0.001)) * 0.5 + 0.5;
    float edge_distance = min(min(local.x, local.y), min(1.0 - local.x, 1.0 - local.y));
    float skirt_band = (1.0 - smoothstep(0.012, 0.045, edge_distance)) * pos_z_face;
    color = mix(color, vec3(1.0, 0.66, 0.16), skirt_band * u_seam_skirts * 0.88);

    // Show the bounded four-child split of the selected Pos Z patch.
    float cross_x = 1.0 - smoothstep(0.0, 0.012, abs(local.x - 0.5));
    float cross_y = 1.0 - smoothstep(0.0, 0.012, abs(local.y - 0.5));
    color = mix(color, vec3(1.0, 0.87, 0.38), max(cross_x, cross_y) * u_split_enabled * 0.75);

    out_color = vec4(color, 1.0);
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shader_sources_declare_boundary_and_skirt_uniforms() {
        assert!(FRAGMENT_SHADER.contains("u_boundary_latitude"));
        assert!(FRAGMENT_SHADER.contains("u_seam_skirts"));
        assert!(FRAGMENT_SHADER.contains("u_split_enabled"));
    }
}
