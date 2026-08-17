use crate::sphere_world_shader::{ShaderUniforms, SphereWorldShader};
use eframe::egui::{self, Color32, Pos2, RichText, Stroke};
use sphere_world_core::{
    compile_patch, cube_sphere_direction, diagnostics, observer_frame, split_patch,
    structure_transform, PatchMesh, WorldDiagnostics,
};
use sphere_world_schema::{
    accepted, validate_world, CubeFace, DerivedOutputs, Layer, LodPolicy, ObserverAnchor, PatchId,
    ProjectionKind, SphereWorld, StructureFootprint, Topology, TopologyKind, ValidationCheck, Vec3,
    WORLD_SCHEMA,
};

pub struct SphereWorldLab {
    world: SphereWorld,
    selected_patch: PatchId,
    root_mesh: Option<PatchMesh>,
    split_meshes: Vec<PatchMesh>,
    validation: Vec<ValidationCheck>,
    diagnostics: Option<WorldDiagnostics>,
    manifest_json: String,
    error: Option<String>,
    show_wireframe: bool,
    show_split: bool,
    yaw: f32,
    pitch: f32,
    zoom: f32,
    shader: Option<SphereWorldShader>,
    shader_error: Option<String>,
}

impl Default for SphereWorldLab {
    fn default() -> Self {
        Self::new(None)
    }
}

impl SphereWorldLab {
    pub fn new(gl: Option<std::sync::Arc<egui_glow::glow::Context>>) -> Self {
        let (shader, shader_error) = match gl {
            Some(context) => match SphereWorldShader::new(context) {
                Ok(shader) => (Some(shader), None),
                Err(error) => (None, Some(error)),
            },
            None => (
                None,
                Some("Glow renderer unavailable; using CPU reference fallback.".into()),
            ),
        };
        let mut lab = Self {
            world: sample_world(),
            selected_patch: PatchId::root(CubeFace::PosZ),
            root_mesh: None,
            split_meshes: Vec::new(),
            validation: Vec::new(),
            diagnostics: None,
            manifest_json: String::new(),
            error: None,
            show_wireframe: true,
            show_split: false,
            yaw: 0.0,
            pitch: 0.18,
            zoom: 1.0,
            shader,
            shader_error,
        };
        lab.rebuild();
        lab
    }
    pub fn ui(&mut self, ctx: &egui::Context) {
        let mut changed = false;
        egui::SidePanel::left("sphereworld-controls")
            .resizable(true)
            .default_width(294.0)
            .show(ctx, |ui| {
                ui.heading("SphereWorld controls");
                ui.small("Edit canonical values. Geometry, wireframes, and diagnostics are rebuilt artifacts.");
                ui.add_space(8.0);

                ui.label(RichText::new("Canonical sphere").strong());
                changed |= ui
                    .add(
                        egui::DragValue::new(&mut self.world.radius_m)
                            .speed(5.0)
                            .clamp_range(50.0..=100_000.0)
                            .prefix("radius: ")
                            .suffix(" m"),
                    )
                    .changed();
                changed |= ui
                    .add(egui::DragValue::new(&mut self.world.seed).speed(1.0).prefix("seed: "))
                    .changed();

                ui.add_space(8.0);
                ui.label(RichText::new("Procedural terrain").strong());
                let mut amplitude = radial_amplitude(&self.world);
                if ui
                    .add(
                        egui::Slider::new(&mut amplitude, 0.0..=250.0)
                            .text("relief amplitude (m)"),
                    )
                    .changed()
                {
                    set_radial_amplitude(&mut self.world, amplitude);
                    changed = true;
                }
                let mut frequency = radial_frequency(&self.world);
                if ui
                    .add(
                        egui::Slider::new(&mut frequency, 0.0005..=0.03)
                            .logarithmic(true)
                            .text("relief frequency"),
                    )
                    .changed()
                {
                    set_radial_frequency(&mut self.world, frequency);
                    changed = true;
                }
                let mut blocked_latitude = boundary_latitude(&self.world);
                if ui
                    .add(
                        egui::Slider::new(&mut blocked_latitude, 0.0..=89.0)
                            .text("blocked polar band starts"),
                    )
                    .changed()
                {
                    set_boundary_latitude(&mut self.world, blocked_latitude);
                    changed = true;
                }

                ui.add_space(8.0);
                ui.label(RichText::new("Patch derivation").strong());
                let mut resolution = self.world.topology.patch_resolution as u32;
                if ui
                    .add(
                        egui::Slider::new(&mut resolution, 3..=65)
                            .step_by(2.0)
                            .text("grid resolution"),
                    )
                    .changed()
                {
                    self.world.topology.patch_resolution = if resolution % 2 == 0 {
                        (resolution + 1).min(65) as u16
                    } else {
                        resolution as u16
                    };
                    changed = true;
                }
                changed |= ui.checkbox(&mut self.world.lod.use_edge_skirts, "generate seam skirts").changed();
                changed |= ui.checkbox(&mut self.show_wireframe, "show wireframe").changed();
                if ui.checkbox(&mut self.show_split, "split selected Pos Z patch").changed() {
                    changed = true;
                }

                ui.add_space(8.0);
                ui.label(RichText::new("Observation camera").strong());
                let anchor = &mut self.world.anchors[0];
                changed |= ui
                    .add(egui::Slider::new(&mut anchor.altitude_m, 0.0..=100.0).text("altitude (m)"))
                    .changed();
                changed |= ui
                    .add(
                        egui::Slider::new(&mut anchor.heading_degrees, -180.0..=180.0)
                            .text("heading (degrees)"),
                    )
                    .changed();
                changed |= ui
                    .add(
                        egui::Slider::new(&mut anchor.u, 0.0..=1.0)
                            .text("surface u (Pos Z)"),
                    )
                    .changed();
                changed |= ui
                    .add(
                        egui::Slider::new(&mut anchor.v, 0.0..=1.0)
                            .text("surface v (Pos Z)"),
                    )
                    .changed();

                ui.add_space(12.0);
                if ui.button("Rebuild derived world").clicked() {
                    changed = true;
                }
                if ui.button("Reset in-memory sample").clicked() {
                    self.world = sample_world();
                    self.selected_patch = PatchId::root(CubeFace::PosZ);
                    self.show_split = false;
                    self.yaw = 0.0;
                    self.pitch = 0.18;
                    self.zoom = 1.0;
                    changed = true;
                }
                if ui.button("Load checked fixture").clicked() {
                    match checked_fixture() {
                        Ok(world) => {
                            self.world = world;
                            self.selected_patch = PatchId::root(CubeFace::PosZ);
                            self.show_split = false;
                            changed = true;
                        }
                        Err(error) => self.error = Some(format!("Fixture load failed: {error}")),
                    }
                }
                ui.separator();
                ui.small("This is a reference manipulation lab. It does not claim literal holographic or 5D behavior, and it does not persist generated mesh buffers as world truth.");
            });

        if changed {
            self.rebuild();
        }

        egui::SidePanel::right("sphereworld-evidence")
            .resizable(true)
            .default_width(328.0)
            .show(ctx, |ui| {
                ui.heading("Derived-world evidence");
                if let Some(error) = &self.error {
                    ui.colored_label(Color32::from_rgb(255, 131, 131), error);
                }
                if let Some(shader_error) = &self.shader_error {
                    ui.colored_label(Color32::from_rgb(255, 191, 118), shader_error);
                } else {
                    ui.colored_label(
                        Color32::from_rgb(122, 226, 165),
                        "GPU shader: real-time Glow boundary/skirt visualization",
                    );
                }
                if let Some(diagnostics) = &self.diagnostics {
                    ui.label(format!(
                        "World digest: {}…",
                        &diagnostics.world_digest[..16]
                    ));
                    ui.label(format!(
                        "Active patches: {}",
                        diagnostics.active_patch_count
                    ));
                    ui.label(format!(
                        "Patch resolution: {} × {}",
                        diagnostics.patch_resolution, diagnostics.patch_resolution
                    ));
                    if let Some(mesh) = &self.root_mesh {
                        ui.label(format!("Selected patch vertices: {}", mesh.vertices.len()));
                        ui.label(format!(
                            "Selected patch triangles: {}",
                            mesh.triangles.len()
                        ));
                        ui.label(format!("Skirt triangles: {}", mesh.skirt_triangle_count));
                        ui.label(format!("Patch digest: {}…", &mesh.digest[..16]));
                    }
                    ui.add_space(8.0);
                    ui.label(RichText::new("Manifest validation").strong());
                    for check in &self.validation {
                        let color = if check.passed {
                            Color32::from_rgb(122, 226, 165)
                        } else {
                            Color32::from_rgb(255, 131, 131)
                        };
                        ui.colored_label(
                            color,
                            format!("{}  {}", if check.passed { "✓" } else { "×" }, check.name),
                        );
                        ui.small(&check.detail);
                    }
                    ui.add_space(10.0);
                    if let Some(anchor) = self.world.anchors.first() {
                        let frame = observer_frame(&self.world, anchor);
                        ui.label(RichText::new("Observation anchor").strong());
                        ui.small(format!(
                            "origin  ({:.1}, {:.1}, {:.1})",
                            frame.origin.x, frame.origin.y, frame.origin.z
                        ));
                        ui.small(format!(
                            "surface up  ({:.2}, {:.2}, {:.2})",
                            frame.up.x, frame.up.y, frame.up.z
                        ));
                    }
                    if let Some(structure) = self.world.structures.first() {
                        let transform = structure_transform(&self.world, structure);
                        ui.add_space(8.0);
                        ui.label(RichText::new("Structure footprint").strong());
                        ui.small(format!(
                            "{}  {:.1} m × {:.1} m",
                            transform.id, transform.width_m, transform.depth_m
                        ));
                    }
                } else {
                    ui.label("No valid derived output is available.");
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("SphereWorld Lab");
            ui.small("Drag the viewport to rotate the derived sphere. Scroll to zoom. Cyan is walkable terrain; coral is the blocked boundary. The gold pin is the active observation point.");
            ui.add_space(6.0);
            self.draw_viewport(ui);
            ui.add_space(8.0);
            egui::CollapsingHeader::new("Canonical sphere-world manifest")
                .default_open(false)
                .show(ui, |ui| {
                    egui::ScrollArea::vertical().max_height(220.0).show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut self.manifest_json)
                                .font(egui::TextStyle::Monospace)
                                .interactive(false)
                                .desired_width(f32::INFINITY),
                        );
                    });
                });
        });
    }

    fn rebuild(&mut self) {
        self.validation = validate_world(&self.world);
        self.root_mesh = None;
        self.split_meshes.clear();
        self.diagnostics = None;
        self.error = None;
        self.manifest_json = serde_json::to_string_pretty(&self.world)
            .unwrap_or_else(|error| format!("Manifest serialization failed: {error}"));

        if !accepted(&self.validation) {
            self.error =
                Some("Manifest validation failed; derived patches were not rebuilt.".into());
            return;
        }
        match compile_patch(&self.world, self.selected_patch) {
            Ok(mesh) => self.root_mesh = Some(mesh),
            Err(error) => {
                self.error = Some(format!("Patch build failed: {error}"));
                return;
            }
        }
        if self.show_split {
            match split_patch(&self.world, self.selected_patch) {
                Ok(patches) => self.split_meshes = patches,
                Err(error) => {
                    self.error = Some(format!("Patch split failed: {error}"));
                    return;
                }
            }
        }
        let active_count = 6 + if self.show_split { 4 } else { 0 };
        match diagnostics(&self.world, active_count) {
            Ok(value) => self.diagnostics = Some(value),
            Err(error) => self.error = Some(format!("Diagnostics failed: {error}")),
        }
    }

    fn draw_viewport(&mut self, ui: &mut egui::Ui) {
        let desired = egui::vec2(ui.available_width(), 390.0);
        let (rect, response) = ui.allocate_exact_size(desired, egui::Sense::drag());
        if response.dragged() {
            let delta = response.drag_delta();
            self.yaw += delta.x * 0.008;
            self.pitch = (self.pitch + delta.y * 0.008).clamp(-1.25, 1.25);
        }
        if response.hovered() {
            let scroll = ui.input(|input| input.raw_scroll_delta.y);
            if scroll.abs() > f32::EPSILON {
                self.zoom = (self.zoom * (1.0 + scroll * 0.0015)).clamp(0.55, 2.25);
            }
        }

        let boundary = boundary_latitude(&self.world) as f32;
        let shader_active = if let Some(shader) = &self.shader {
            shader.update(ShaderUniforms {
                boundary_latitude_degrees: boundary,
                seam_skirts_enabled: self.world.lod.use_edge_skirts,
                split_enabled: self.show_split,
                yaw: self.yaw,
                pitch: self.pitch,
                zoom: self.zoom,
            });
            shader.paint(ui, rect);
            true
        } else {
            false
        };

        let painter = ui.painter_at(rect);
        if !shader_active {
            painter.rect_filled(rect, 10.0, Color32::from_rgb(9, 16, 27));
            let center = rect.center();
            let radius = rect.width().min(rect.height()) * 0.38 * self.zoom;
            painter.circle_filled(center, radius, Color32::from_rgb(15, 37, 58));
            painter.circle_stroke(
                center,
                radius,
                Stroke::new(2.0_f32, Color32::from_rgb(91, 145, 191)),
            );
        }
        painter.rect_stroke(
            rect,
            10.0,
            Stroke::new(1.0_f32, Color32::from_rgb(48, 72, 102)),
        );

        if let Some(mesh) = &self.root_mesh {
            self.draw_mesh(&painter, rect, mesh, Color32::from_rgb(105, 213, 192), 1.0);
        }
        for mesh in &self.split_meshes {
            self.draw_mesh(&painter, rect, mesh, Color32::from_rgb(255, 188, 104), 0.6);
        }
        if let Some(anchor) = self.world.anchors.first() {
            let direction = cube_sphere_direction(anchor.face, anchor.u, anchor.v);
            if let Some(position) = self.project(direction, rect) {
                painter.circle_filled(position, 7.0, Color32::from_rgb(255, 219, 119));
                painter.circle_stroke(position, 7.0, Stroke::new(1.5_f32, Color32::WHITE));
                painter.text(
                    egui::pos2(position.x + 10.0, position.y - 8.0),
                    egui::Align2::LEFT_BOTTOM,
                    "observer",
                    egui::FontId::proportional(12.0),
                    Color32::from_rgb(255, 231, 162),
                );
            }
        }
        if let Some(footprint) = self.world.structures.first() {
            let direction = cube_sphere_direction(footprint.face, footprint.u, footprint.v);
            if let Some(position) = self.project(direction, rect) {
                painter.rect_filled(
                    egui::Rect::from_center_size(position, egui::vec2(11.0, 11.0)),
                    2.0,
                    Color32::from_rgb(212, 152, 88),
                );
            }
        }
        painter.text(
            egui::pos2(rect.left() + 12.0, rect.top() + 12.0),
            egui::Align2::LEFT_TOP,
            "Front-facing samples of Pos Z patch; drag to rotate • scroll to zoom",
            egui::FontId::monospace(12.0),
            Color32::from_rgb(193, 214, 239),
        );
    }

    fn draw_mesh(
        &self,
        painter: &egui::Painter,
        rect: egui::Rect,
        mesh: &PatchMesh,
        base_color: Color32,
        alpha: f32,
    ) {
        let base_vertex_count = (self.world.topology.patch_resolution as usize).pow(2);
        for vertex in mesh.vertices.iter().take(base_vertex_count) {
            if let Some(position) = self.project(vertex.direction, rect) {
                let color = if vertex.walkable {
                    base_color
                } else {
                    Color32::from_rgb(240, 113, 104)
                };
                painter.circle_filled(position, 1.4, color.gamma_multiply(alpha));
            }
        }
        if self.show_wireframe {
            for segment in &mesh.wire_segments {
                let a = &mesh.vertices[segment[0] as usize];
                let b = &mesh.vertices[segment[1] as usize];
                if let (Some(a), Some(b)) = (
                    self.project(a.direction, rect),
                    self.project(b.direction, rect),
                ) {
                    painter.line_segment(
                        [a, b],
                        Stroke::new(0.65_f32, base_color.gamma_multiply(alpha * 0.68)),
                    );
                }
            }
        }
    }

    fn project(&self, direction: Vec3, rect: egui::Rect) -> Option<Pos2> {
        let yaw_sin = self.yaw.sin();
        let yaw_cos = self.yaw.cos();
        let pitch_sin = self.pitch.sin();
        let pitch_cos = self.pitch.cos();
        let x = direction.x as f32 * yaw_cos + direction.z as f32 * yaw_sin;
        let z = -direction.x as f32 * yaw_sin + direction.z as f32 * yaw_cos;
        let y = direction.y as f32 * pitch_cos - z * pitch_sin;
        let depth = direction.y as f32 * pitch_sin + z * pitch_cos;
        if depth < -0.08 {
            return None;
        }
        let center = rect.center();
        let radius = rect.width().min(rect.height()) * 0.38 * self.zoom;
        Some(egui::pos2(center.x + x * radius, center.y - y * radius))
    }
}

fn checked_fixture() -> Result<SphereWorld, serde_json::Error> {
    serde_json::from_str(include_str!(
        "../../../examples/sphere-world-basic/world.sphereworld.json"
    ))
}

fn sample_world() -> SphereWorld {
    SphereWorld {
        schema: WORLD_SCHEMA.into(),
        world_id: "sphereworld-lab".into(),
        seed: 42,
        radius_m: 1000.0,
        topology: Topology {
            kind: TopologyKind::CubeSphere,
            projection: ProjectionKind::NormalizedCube,
            max_level: 4,
            patch_resolution: 33,
        },
        lod: LodPolicy {
            max_neighbor_level_delta: 1,
            use_edge_skirts: true,
        },
        layers: vec![
            Layer::RadialNoise {
                id: "base-relief".into(),
                amplitude_m: 35.0,
                frequency: 0.004,
                seed_offset: 0,
            },
            Layer::LatitudeBand {
                id: "polar-boundary".into(),
                min_degrees: 70.0,
                max_degrees: 90.0,
                behavior: "blocked".into(),
            },
        ],
        anchors: vec![ObserverAnchor {
            id: "player-start".into(),
            kind: "camera_observer".into(),
            face: CubeFace::PosZ,
            level: 0,
            u: 0.5,
            v: 0.5,
            altitude_m: 2.0,
            heading_degrees: 0.0,
            pitch_degrees: -10.0,
        }],
        structures: vec![StructureFootprint {
            id: "observation-platform".into(),
            face: CubeFace::PosZ,
            level: 0,
            u: 0.62,
            v: 0.50,
            width_m: 12.0,
            depth_m: 8.0,
            heading_degrees: 15.0,
        }],
        derived_outputs: DerivedOutputs {
            render_mesh: true,
            wireframe: true,
            collision_mesh: true,
        },
    }
}

fn radial_amplitude(world: &SphereWorld) -> f64 {
    world
        .layers
        .iter()
        .find_map(|layer| match layer {
            Layer::RadialNoise { amplitude_m, .. } => Some(*amplitude_m),
            _ => None,
        })
        .unwrap_or(0.0)
}

fn radial_frequency(world: &SphereWorld) -> f64 {
    world
        .layers
        .iter()
        .find_map(|layer| match layer {
            Layer::RadialNoise { frequency, .. } => Some(*frequency),
            _ => None,
        })
        .unwrap_or(0.004)
}

fn set_radial_amplitude(world: &mut SphereWorld, value: f64) {
    if let Some(Layer::RadialNoise { amplitude_m, .. }) = world
        .layers
        .iter_mut()
        .find(|layer| matches!(layer, Layer::RadialNoise { .. }))
    {
        *amplitude_m = value;
    }
}

fn set_radial_frequency(world: &mut SphereWorld, value: f64) {
    if let Some(Layer::RadialNoise { frequency, .. }) = world
        .layers
        .iter_mut()
        .find(|layer| matches!(layer, Layer::RadialNoise { .. }))
    {
        *frequency = value;
    }
}

fn boundary_latitude(world: &SphereWorld) -> f64 {
    world
        .layers
        .iter()
        .find_map(|layer| match layer {
            Layer::LatitudeBand { min_degrees, .. } => Some(*min_degrees),
            _ => None,
        })
        .unwrap_or(70.0)
}

fn set_boundary_latitude(world: &mut SphereWorld, value: f64) {
    if let Some(Layer::LatitudeBand {
        min_degrees,
        max_degrees,
        ..
    }) = world
        .layers
        .iter_mut()
        .find(|layer| matches!(layer, Layer::LatitudeBand { .. }))
    {
        *min_degrees = value;
        *max_degrees = 90.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checked_fixture_loads_and_passes_manifest_validation() {
        let world = checked_fixture().expect("checked SphereWorld fixture should deserialize");
        assert!(accepted(&validate_world(&world)));
        assert_eq!(world.world_id, "sphere-world-basic");
    }
}
