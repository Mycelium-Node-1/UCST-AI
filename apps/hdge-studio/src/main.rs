use eframe::egui::{self, Color32, RichText, Stroke};
use hdge_core::{compile_fgl, execute_tdm, sample_scene, Compilation};
use hdge_schema::{canonical_json, PrimitiveKind, TdmRun, Vec3};

mod sphere_world_lab;
use sphere_world_lab::SphereWorldLab;

const SAMPLE_SOURCE: &str = "☉⊗∆⚘∎";

struct StudioApp {
    source: String,
    scene_id: String,
    compilation: Option<Compilation>,
    tdm_run: Option<TdmRun>,
    scene_json: String,
    error: Option<String>,
    selected_debug: DebugView,
    selected_workspace: Workspace,
    sphere_world_lab: SphereWorldLab,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum DebugView {
    World,
    Distance,
    Validation,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Workspace {
    Fgl,
    SphereWorld,
}

impl Default for StudioApp {
    fn default() -> Self {
        let mut app = Self {
            source: SAMPLE_SOURCE.into(),
            scene_id: "life-seed".into(),
            compilation: None,
            tdm_run: None,
            scene_json: String::new(),
            error: None,
            selected_debug: DebugView::World,
            selected_workspace: Workspace::Fgl,
            sphere_world_lab: SphereWorldLab::default(),
        };
        app.compile_and_run();
        app
    }
}

impl StudioApp {
    fn compile_and_run(&mut self) {
        match compile_fgl(&self.source, &self.scene_id) {
            Ok(compilation) => {
                self.scene_json = canonical_json(&compilation.scene)
                    .and_then(|json| {
                        serde_json::from_str::<serde_json::Value>(&json)
                            .map_err(hdge_schema::ContractError::from)
                            .and_then(|value| {
                                serde_json::to_string_pretty(&value)
                                    .map_err(hdge_schema::ContractError::from)
                            })
                    })
                    .unwrap_or_else(|error| format!("Serialization error: {error}"));
                self.tdm_run = Some(execute_tdm(&compilation, "uncommitted"));
                self.compilation = Some(compilation);
                self.error = None;
            }
            Err(error) => {
                self.compilation = None;
                self.tdm_run = None;
                self.scene_json.clear();
                self.error = Some(error.to_string());
            }
        }
    }

    fn reset_fixture(&mut self) {
        self.source = SAMPLE_SOURCE.into();
        self.scene_id = "life-seed".into();
        self.compile_and_run();
    }

    fn draw_viewport(&self, ui: &mut egui::Ui) {
        let desired = egui::vec2(ui.available_width(), 300.0);
        let (rect, _) = ui.allocate_exact_size(desired, egui::Sense::hover());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 10.0, Color32::from_rgb(12, 18, 29));
        painter.rect_stroke(
            rect,
            10.0,
            Stroke::new(1.0_f32, Color32::from_rgb(55, 79, 108)),
        );

        let center = rect.center();
        let grid_color = Color32::from_rgb(31, 50, 73);
        for step in -4..=4 {
            let x = center.x + step as f32 * 34.0;
            let y = center.y + step as f32 * 34.0;
            painter.line_segment(
                [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                Stroke::new(1.0_f32, grid_color),
            );
            painter.line_segment(
                [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                Stroke::new(1.0_f32, grid_color),
            );
        }
        painter.line_segment(
            [
                egui::pos2(rect.left(), center.y),
                egui::pos2(rect.right(), center.y),
            ],
            Stroke::new(1.4_f32, Color32::from_rgb(78, 109, 148)),
        );
        painter.line_segment(
            [
                egui::pos2(center.x, rect.top()),
                egui::pos2(center.x, rect.bottom()),
            ],
            Stroke::new(1.4_f32, Color32::from_rgb(78, 109, 148)),
        );

        if let Some(compilation) = &self.compilation {
            for primitive in &compilation.scene.primitives {
                let position = egui::pos2(
                    center.x + (primitive.center.x as f32 * 62.0),
                    center.y - (primitive.center.y as f32 * 62.0),
                );
                match primitive.kind {
                    PrimitiveKind::Sphere => {
                        let radius = primitive.radius.unwrap_or(0.25) as f32 * 62.0;
                        let fill = if primitive.id.contains("anchor") {
                            Color32::from_rgb(96, 175, 255)
                        } else {
                            Color32::from_rgb(113, 222, 165)
                        };
                        painter.circle_filled(position, radius, fill.gamma_multiply(0.62));
                        painter.circle_stroke(position, radius, Stroke::new(2.0_f32, fill));
                        painter.text(
                            position,
                            egui::Align2::CENTER_CENTER,
                            primitive.source_symbol.as_deref().unwrap_or("•"),
                            egui::FontId::proportional(20.0),
                            Color32::WHITE,
                        );
                    }
                    PrimitiveKind::Box => {
                        let half = primitive.half_extents.unwrap_or(Vec3 {
                            x: 0.5,
                            y: 0.5,
                            z: 0.5,
                        });
                        let box_rect = egui::Rect::from_center_size(
                            position,
                            egui::vec2((half.x * 2.0) as f32 * 62.0, (half.y * 2.0) as f32 * 62.0),
                        );
                        painter.rect_filled(box_rect, 4.0, Color32::from_rgb(192, 143, 85));
                        painter.rect_stroke(
                            box_rect,
                            4.0,
                            Stroke::new(2.0_f32, Color32::from_rgb(244, 194, 118)),
                        );
                    }
                    PrimitiveKind::Plane => {
                        painter.line_segment(
                            [
                                egui::pos2(rect.left(), position.y),
                                egui::pos2(rect.right(), position.y),
                            ],
                            Stroke::new(3.0_f32, Color32::from_rgb(155, 164, 180)),
                        );
                    }
                }
                painter.text(
                    egui::pos2(position.x, position.y + 74.0),
                    egui::Align2::CENTER_TOP,
                    &primitive.id,
                    egui::FontId::proportional(12.0),
                    Color32::from_rgb(196, 214, 233),
                );
            }

            if self.selected_debug == DebugView::Distance {
                let probe = sample_scene(&compilation.scene, Vec3::ZERO);
                painter.text(
                    egui::pos2(rect.left() + 12.0, rect.top() + 12.0),
                    egui::Align2::LEFT_TOP,
                    format!(
                        "SDF probe at origin: {:.3} ({})",
                        probe.distance,
                        probe.primitive_id.unwrap_or_else(|| "none".into())
                    ),
                    egui::FontId::monospace(13.0),
                    Color32::from_rgb(255, 221, 132),
                );
            }
        } else {
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "Compile a supported FGL clause to inspect the declarative scene.",
                egui::FontId::proportional(16.0),
                Color32::from_rgb(182, 197, 216),
            );
        }
    }
}

impl eframe::App for StudioApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading(RichText::new("HDGE Studio").color(Color32::from_rgb(133, 206, 255)));
                ui.separator();
                ui.label("Workspace:");
                ui.selectable_value(
                    &mut self.selected_workspace,
                    Workspace::Fgl,
                    "FGL Workbench",
                );
                ui.selectable_value(
                    &mut self.selected_workspace,
                    Workspace::SphereWorld,
                    "SphereWorld Lab",
                );
                if self.selected_workspace == Workspace::Fgl {
                    ui.separator();
                    if ui.button("Compile + run TDM").clicked() {
                        self.compile_and_run();
                    }
                    if ui.button("Reset sample").clicked() {
                        self.reset_fixture();
                    }
                    ui.separator();
                    ui.label("View:");
                    ui.selectable_value(&mut self.selected_debug, DebugView::World, "World");
                    ui.selectable_value(
                        &mut self.selected_debug,
                        DebugView::Distance,
                        "Distance probe",
                    );
                    ui.selectable_value(
                        &mut self.selected_debug,
                        DebugView::Validation,
                        "Validation",
                    );
                }
            });
        });

        if self.selected_workspace == Workspace::SphereWorld {
            self.sphere_world_lab.ui(ctx);
            return;
        }

        egui::SidePanel::left("source").resizable(true).default_width(280.0).show(ctx, |ui| {
            ui.heading("Declarative source");
            ui.label("Documented FGL subset: ☉ Ϟ ⊗ ⚘ ⟡ ∆ ⟁ ✶");
            ui.add_space(8.0);
            ui.label("Scene identifier");
            ui.text_edit_singleline(&mut self.scene_id);
            ui.add_space(8.0);
            ui.label("FGL clause");
            ui.add(egui::TextEdit::multiline(&mut self.source).desired_rows(7).code_editor());
            if let Some(error) = &self.error {
                ui.add_space(10.0);
                ui.colored_label(Color32::from_rgb(255, 131, 131), format!("Compiler error: {error}"));
            }
            ui.add_space(12.0);
            ui.separator();
            ui.label(RichText::new("Scope").strong());
            ui.small("This application implements an explicit, inspectable reference mapping for a documented FGL subset. It is a declarative SDF-workbench prototype, not a literal physical holographic or 5D engine.");
        });

        egui::SidePanel::right("evidence")
            .resizable(true)
            .default_width(310.0)
            .show(ctx, |ui| {
                ui.heading("Execution evidence");
                if let Some(compilation) = &self.compilation {
                    ui.label(format!(
                        "Scene digest: {}…",
                        &compilation.scene_digest[..16]
                    ));
                    ui.label(format!(
                        "Source symbols: {}",
                        compilation
                            .source
                            .symbols
                            .iter()
                            .map(|item| item.symbol)
                            .collect::<String>()
                    ));
                    ui.add_space(8.0);
                    ui.label(RichText::new("Validation").strong());
                    for check in &compilation.validation {
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
                    ui.add_space(8.0);
                    ui.label(RichText::new("TDM run").strong());
                    if let Some(run) = &self.tdm_run {
                        for event in &run.events {
                            ui.label(format!("{:02}  {:?}", event.tick, event.phase));
                            ui.small(&event.detail);
                        }
                        let outcome = if run.accepted {
                            "Eligible for commit"
                        } else {
                            "Rollback required"
                        };
                        ui.colored_label(
                            if run.accepted {
                                Color32::from_rgb(122, 226, 165)
                            } else {
                                Color32::from_rgb(255, 131, 131)
                            },
                            outcome,
                        );
                    }
                } else {
                    ui.label("No accepted compilation is available.");
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Analytic scene viewport");
            ui.small("CPU reference visualization of the compiled Scene IR. The MM3E adapter will replace this inspection layer after parity gates pass.");
            ui.add_space(6.0);
            self.draw_viewport(ui);
            ui.add_space(10.0);
            egui::CollapsingHeader::new("Canonical Scene IR").default_open(false).show(ui, |ui| {
                egui::ScrollArea::vertical().max_height(220.0).show(ui, |ui| {
                    ui.add(egui::TextEdit::multiline(&mut self.scene_json).font(egui::TextStyle::Monospace).desired_width(f32::INFINITY));
                });
            });
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 760.0])
            .with_min_inner_size([900.0, 620.0]),
        ..Default::default()
    };
    eframe::run_native(
        "HDGE Studio",
        options,
        Box::new(|_creation_context| Box::<StudioApp>::default()),
    )
}
