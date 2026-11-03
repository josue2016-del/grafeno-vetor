//! Assets Library docker — browse and insert assets into the drawing.
//!
//! Scans the `assets/` directory next to the source tree and shows images
//! as thumbnails. Clicking one imports it onto the active layer.

use crate::app::App;
use crate::i18n::tr;
use egui::{RichText, Ui, Vec2};
use std::path::PathBuf;

/// State for the Assets Library docker tab.
#[derive(Default)]
pub struct AssetsLibraryState {
    /// Whether the library panel is expanded.
    pub expanded: bool,
    /// Paths discovered so far (empty = need refresh).
    pub paths: Vec<PathBuf>,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn assets_dir() -> Option<PathBuf> {
    // Walk up from the binary location or use the known assets folder.
    let candidate = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("assets");
    if candidate.exists() {
        Some(candidate)
    } else {
        None
    }
}

fn image_extensions() -> &'static [&'static str] {
    &["png", "jpg", "jpeg", "webp", "svg", "bmp"]
}

fn discover_assets(dir: &std::path::Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if !dir.is_dir() {
        return out;
    }
    for entry in match dir.read_dir() {
        Ok(r) => r,
        Err(_) => return out,
    } {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if image_extensions().contains(&ext.to_lowercase().as_str()) {
                    out.push(path);
                }
            }
        }
    }
    out.sort();
    out
}

// ---------------------------------------------------------------------------
// Public entry
// ---------------------------------------------------------------------------

pub fn show(app: &mut App, ui: &mut Ui) {
    let state = app.assets_library_state.get_or_insert_with(AssetsLibraryState::default);
    let Some(assets_dir) = assets_dir() else {
        ui.label(
            RichText::new(tr("assets_library.title"))
                .size(12.0)
                .strong(),
        );
        ui.add_space(4.0);
        ui.label(
            RichText::new(tr("assets_library.no_dir"))
                .size(10.0)
                .color(egui::Color32::from_rgb(0x88, 0x88, 0x88)),
        );
        return;
    };

    // Refresh button.
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(tr("assets_library.title"))
                .size(12.0)
                .strong(),
        );
        ui.add_space(8.0);
        ui.separator();
        ui.add_space(4.0);
        if ui.small_button(tr("assets_library.refresh")).clicked() {
            state.paths = discover_assets(&assets_dir);
            state.expanded = true;
        }
    });
    ui.add_space(4.0);

    // If not expanded, just show a hint.
    if !state.expanded {
        ui.label(
            RichText::new(tr("assets_library.hint"))
                .size(10.0)
                .color(egui::Color32::from_rgb(0x88, 0x88, 0x88)),
        );
        return;
    }

    // Discover if empty.
    if state.paths.is_empty() {
        state.paths = discover_assets(&assets_dir);
    }

    if state.paths.is_empty() {
        ui.label(
            RichText::new(tr("assets_library.empty"))
                .size(10.0)
                .color(egui::Color32::from_rgb(0x88, 0x88, 0x88)),
        );
        return;
    }

    // Grid of thumbnails.
    let item_size = Vec2::splat(80.0);
    let mut imported_path: Option<PathBuf> = None;
    egui::Grid::new("assets_grid")
        .num_columns((ui.available_width() / item_size.x).max(1.0) as usize)
        .spacing([4.0, 4.0])
        .show(ui, |ui| {
            for path in &state.paths {
                ui.horizontal(|ui| {
                    let name = path
                        .file_stem()
                        .and_then(|n| n.to_str())
                        .unwrap_or("");
                    let btn = ui
                        .add_sized(
                            item_size,
                            egui::Button::new(
                                RichText::new(name).size(9.0),
                            ),
                        )
                        .on_hover_text(path.display().to_string());
                    if btn.clicked() {
                        imported_path = Some(path.clone());
                    }
                    ui.end_row();
                });
            }
        });
    let imported = imported_path.clone();
    let count = state.paths.len();
    drop(state);
    if let Some(p) = imported {
        app.import_bitmap(&p);
    }

    ui.add_space(4.0);
    ui.label(
        RichText::new(format!(
            "{} {}",
            tr("assets_library.count"),
            count
        ))
        .size(9.0)
        .color(egui::Color32::from_rgb(0x88, 0x88, 0x88)),
    );
}
