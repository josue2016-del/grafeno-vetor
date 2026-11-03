//! Python IDE docker — integrated editor for Python macros inside Grafeno Vetor.
//!
//! Features:
//!  - Left panel: script list (load from disk, new, save, delete).
//!  - Center: monospace text editor.
//!  - Bottom: console / output panel.
//!  - Toolbar: Run (F5), Save, New, Open file.
//!
//! Keyboard shortcuts:
//!  - `F5` – run current script
//!  - `Ctrl+S` – save current script
//!  - `Ctrl+N` – new script
//!  - `Ctrl+O` – open script
//!  - `Ctrl+D` – delete script

use crate::app::App;
use crate::i18n::tr;
use crate::python_macros::{self, PythonScript, PythonState};
use egui::{Color32, FontId, RichText, ScrollArea, TextEdit, Ui, Vec2};

/// Height of the console panel.
const CONSOLE_H: f32 = 140.0;
/// Font for the editor.
const EDITOR_FONT: FontId = FontId::monospace(12.0);

/// State for the Python IDE docker.
#[derive(Default)]
pub struct PythonIdeState {
    /// Index of the currently selected script in `py.scripts`.
    pub selected_idx: Option<usize>,
    /// Source text shown in the editor (synced on selection change).
    pub display_source: String,
}

// ---------------------------------------------------------------------------
// Public entry
// ---------------------------------------------------------------------------

pub fn show(app: &mut App, ui: &mut Ui) {
    let state = app.python_ide_state.get_or_insert_with(PythonIdeState::default);
    let py = &mut app.python_state;
    py.ensure_loaded();
    py.drain_output();

    toolbar(ui, py, state);
    ui.separator();
    body(ui, py, state);
}

// ---------------------------------------------------------------------------
// Toolbar
// ---------------------------------------------------------------------------

fn toolbar(ui: &mut Ui, py: &mut PythonState, state: &mut PythonIdeState) {
    ui.horizontal(|ui| {
        // Python version indicator
        if let Some(ver) = python_macros::python_version() {
            ui.label(
                RichText::new(&ver)
                    .size(10.0)
                    .color(Color32::from_rgb(0x88, 0xAA, 0xFF)),
            );
        } else {
            ui.label(
                RichText::new(tr("python.no_python"))
                    .size(10.0)
                    .color(Color32::RED),
            );
        }

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(4.0);

        // Run
        if ui
            .button(RichText::new("\u{25B6} ").size(12.0).color(Color32::from_rgb(0x44, 0xCC, 0x44)))
            .on_hover_text(tr("python.run"))
            .clicked()
        {
            run_current(py, state);
        }

        // Save
        if ui
            .button("\u{1F4BE}")
            .on_hover_text(tr("python.save"))
            .clicked()
        {
            save_current(py, state);
        }

        // New
        if ui
            .button("\u{1F4C4}")
            .on_hover_text(tr("python.new"))
            .clicked()
        {
            create_new(py, state);
        }

        // Open
        if ui
            .button("\u{1F4C2}")
            .on_hover_text(tr("python.open"))
            .clicked()
        {
            open_script(py, state);
        }

        // Delete
        if ui
            .button("\u{1F5D1}")
            .on_hover_text(tr("python.delete"))
            .clicked()
        {
            delete_current(py, state);
        }

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(4.0);

        // Built-in macros combo
        egui::ComboBox::from_id_salt("builtin_macros")
            .width(170.0)
            .selected_text(tr("python.builtins"))
            .show_ui(ui, |ui| {
                ui.monospace(tr("python.builtins"));
                ui.separator();
                if ui
                    .selectable_label(false, "\u{1F4C5} Calendar Generator")
                    .clicked()
                {
                    state.display_source = python_macros::builtin_calendar().to_string();
                    state.selected_idx = None;
                }
                if ui
                    .selectable_label(false, "\u{27A1} Batch Shapes")
                    .clicked()
                {
                    state.display_source = python_macros::builtin_batch_shape().to_string();
                    state.selected_idx = None;
                }
            });

        ui_spacer(ui);
    });
}

fn ui_spacer(ui: &mut Ui) {
    ui.add_space(6.0);
}

// ---------------------------------------------------------------------------
// Body: left panel + editor/console
// ---------------------------------------------------------------------------

fn body(ui: &mut Ui, py: &mut PythonState, state: &mut PythonIdeState) {
    let avail = ui.available_width();
    let left_w = avail * f32::min(0.28, 200.0 / f32::max(avail, 1.0)).max(0.18);
    let right_w = avail - left_w - 6.0;

    ui.horizontal(|ui| {
        // ── Left: script list ──
        ui.allocate_ui_with_layout(
            Vec2::new(left_w, ui.available_height()),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                ui.heading(RichText::new(tr("python.scripts")).size(12.0));
                ui.add_space(4.0);
                script_list(ui, py, state);
            },
        );

        ui.add_space(6.0);

        // ── Right: editor + console ──
        ui.allocate_ui_with_layout(
            Vec2::new(right_w, ui.available_height()),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| editor_and_console(ui, py, state),
        );
    });
}

// ---------------------------------------------------------------------------
// Script list (left panel)
// ---------------------------------------------------------------------------

fn script_list(
    ui: &mut Ui,
    py: &mut PythonState,
    state: &mut PythonIdeState,
) {
    ScrollArea::vertical()
        .id_salt("python_script_list")
        .show(ui, |ui| {
            for (i, script) in py.scripts.iter().enumerate() {
                let selected = state.selected_idx == Some(i);
                let (_id, rect) = ui.allocate_space(Vec2::new(ui.available_width(), 22.0));
                if ui.interact(rect, egui::Id::new(i), egui::Sense::click()).clicked() {
                    state.selected_idx = Some(i);
                    state.display_source = script.source.clone();
                }
                ui.painter().text(
                    rect.min + egui::vec2(6.0, 6.0),
                    egui::Align2::LEFT_CENTER,
                    format!("\u{1F40D} {}", script.name),
                    FontId::monospace(11.0),
                    if selected {
                        Color32::from_rgb(0x66, 0xCC, 0xFF)
                    } else {
                        Color32::from_rgb(0xDD, 0xDD, 0xDD)
                    },
                );
                if selected {
                    ui.painter().rect_stroke(
                        rect,
                        2.0,
                        egui::Stroke::new(1.0, Color32::from_rgb(0x44, 0xAA, 0xFF)),
                        egui::StrokeKind::Inside,
                    );
                }
            }

            if py.scripts.is_empty() {
                ui.add_space(8.0);
                ui.label(
                    RichText::new(tr("python.no_scripts"))
                        .size(10.0)
                        .color(Color32::from_rgb(0x66, 0x66, 0x66)),
                );
            }
        });
}

// ---------------------------------------------------------------------------
// Editor + Console (right panel)
// ---------------------------------------------------------------------------

fn editor_and_console(
    ui: &mut Ui,
    py: &mut PythonState,
    state: &mut PythonIdeState,
) {
    // Editor
    let editor_h = ui.available_height() * 0.65;
    ui.allocate_ui_with_layout(
        Vec2::new(ui.available_width(), editor_h),
        egui::Layout::top_down(egui::Align::LEFT),
        |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(tr("python.editor_title"))
                        .size(10.0)
                        .color(Color32::from_rgb(0x99, 0xAA, 0xBB)),
                );
                if let Some(idx) = state.selected_idx {
                    if idx < py.scripts.len() {
                        ui.label(
                            RichText::new(&py.scripts[idx].name)
                                .size(10.0)
                                .color(Color32::from_rgb(0x77, 0x88, 0x99)),
                        );
                    }
                }
            });
            ui.add_space(2.0);
            let resp = ui.add(
                TextEdit::multiline(&mut state.display_source)
                    .hint_text(tr("python.editor_hint"))
                    .font(EDITOR_FONT.clone())
                    .lock_focus(true)
                    .desired_width(f32::INFINITY)
                    .desired_rows(18),
            );
            // Auto-save on focus loss
            if resp.lost_focus() {
                if let Some(idx) = state.selected_idx {
                    let _ = py.save_script(idx, &state.display_source);
                }
            }
        },
    );

    // Console
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(tr("python.console")).size(10.0).color(Color32::from_rgb(0x99, 0xAA, 0xBB)),
        );
        if ui.small_button(tr("python.clear")).clicked() {
            py.output.clear();
        }
    });
    ui.add_space(2.0);

    ScrollArea::vertical()
        .id_salt("python_console")
        .show(ui, |ui| {
            // Pull in any async output from the subprocess.
            py.drain_output();
            ui.allocate_ui_with_layout(
                Vec2::new(ui.available_width(), CONSOLE_H),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    for line in &py.output {
                        let is_error =
                            line.contains("Error") || line.contains("error") || line.contains("❌");
                        let is_run = line.contains(">>>") || line.contains("<<");
                        let color = if is_error {
                            Color32::from_rgb(0xFF, 0x66, 0x66)
                        } else if is_run {
                            Color32::from_rgb(0x66, 0xCC, 0x66)
                        } else {
                            Color32::from_rgb(0xCC, 0xCC, 0xCC)
                        };
                        ui.label(RichText::new(line).color(color).size(10.0));
                    }
                    if py.output.is_empty() {
                        ui.label(
                            RichText::new(tr("python.console_hint"))
                                .size(10.0)
                                .color(Color32::from_rgb(0x66, 0x66, 0x66)),
                        );
                    }
                },
            );
        });
}

// ---------------------------------------------------------------------------
// Actions
// ---------------------------------------------------------------------------

fn run_current(py: &mut PythonState, state: &mut PythonIdeState) {
    let source = state.display_source.clone();
    let name = state
        .selected_idx
        .and_then(|i| py.scripts.get(i).map(|s| s.name.clone()))
        .unwrap_or_else(|| "script".to_string());

    python_macros::run_script(py, &source, &name);
}

fn save_current(py: &mut PythonState, state: &mut PythonIdeState) {
    let Some(idx) = state.selected_idx else {
        py.output.push(tr("python.no_selected").to_string());
        return;
    };
    if idx >= py.scripts.len() {
        py.output.push(tr("python.no_selected").to_string());
        return;
    }
    match py.save_script(idx, &state.display_source) {
        Ok(()) => {
            py.output.push(tr("python.saved").to_string());
            py.scripts[idx].source = state.display_source.clone();
        }
        Err(e) => py.output.push(format!("Error saving: {}", e)),
    }
}

fn create_new(py: &mut PythonState, state: &mut PythonIdeState) {
    match py.create_new("NewMacro") {
        Ok(idx) => {
            state.selected_idx = Some(idx);
            state.display_source = py.scripts[idx].source.clone();
            py.output.push(format!("Created: {}", py.scripts[idx].name));
        }
        Err(e) => py.output.push(format!("Error creating: {}", e)),
    }
}

fn open_script(py: &mut PythonState, state: &mut PythonIdeState) {
    use rfd::FileDialog;
    let Some(path) = FileDialog::new()
        .add_filter("Python", &["py"])
        .pick_file()
    else {
        return;
    };
    match std::fs::read_to_string(&path) {
        Ok(source) => {
            let name = path
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "Opened".to_string());
            let idx = py
                .scripts
                .iter()
                .position(|s| s.path == path)
                .unwrap_or_else(|| {
                    py.scripts
                        .push(PythonScript::new(name, path.clone(), source.clone()));
                    py.scripts.len() - 1
                });
            state.selected_idx = Some(idx);
            state.display_source = source;
        }
        Err(e) => py.output.push(format!("Error opening: {}", e)),
    }
}

fn delete_current(py: &mut PythonState, state: &mut PythonIdeState) {
    let Some(idx) = state.selected_idx else {
        return;
    };
    py.delete_script(idx);
    if state.selected_idx == Some(idx) {
        state.selected_idx = None;
        state.display_source.clear();
    } else if let Some(cur) = state.selected_idx {
        if cur > idx {
            state.selected_idx = Some(cur - 1);
        }
    }
}
