//! Importer: when Options > Global > File Formats > "Resize on import" is
//! on (like CorelDRAW), importing an image or file asks for the scale /
//! width and height before placing it. The user confirms with Enter or a
//! mouse click, or cancels.
//!
//! The scale is remembered for the next import.

use crate::app::App;
use crate::i18n::{tr, trf};
use crate::ui::dialogs::{enter_pressed, ok_cancel, window};
use egui::{Context, Slider};
use std::path::PathBuf;
use grafeno_core::Affine;

/// The dialog's state.
#[derive(Debug, Clone, PartialEq)]
pub struct ImportResizeState {
    /// The file being imported.
    pub path: PathBuf,
    /// Scale in percent (100 = original size).
    pub scale: f64,
    /// Width in mm; 0 means "keep aspect ratio".
    pub width_mm: f64,
    /// Height in mm; 0 means "keep aspect ratio".
    pub height_mm: f64,
}

impl Default for ImportResizeState {
    fn default() -> Self {
        Self {
            path: PathBuf::new(),
            scale: 100.0,
            width_mm: 0.0,
            height_mm: 0.0,
        }
    }
}

/// Show the import resize dialog; confirmed via Enter or mouse click.
pub fn dialog(app: &mut App, ctx: &Context, st: &mut ImportResizeState, close: &mut bool) {
    window(ctx, tr("importer.resize_on_import")).show(ctx, |ui| {
        ui.set_max_width(400.0);
        ui.label(trf(
            "importer.resize_confirm",
            &[("name", &st.path.display().to_string())],
        ));
        ui.add_space(12.0);

        // Scale percentage slider.
        ui.label(tr("print.scale"));
        ui.add(Slider::new(&mut st.scale, 1.0..=500.0).suffix(" %"));
        ui.add_space(8.0);

        // Width / height fields in mm.
        ui.horizontal(|ui| {
            ui.label(tr("importer.width_mm"));
            ui.add(
                crate::ui::field::NumField::new(&mut st.width_mm)
                    .range(0.0..=10000.0)
                    .suffix(" mm")
                    .width(100.0),
            );
        });
        ui.horizontal(|ui| {
            ui.label(tr("importer.height_mm"));
            ui.add(
                crate::ui::field::NumField::new(&mut st.height_mm)
                    .range(0.0..=10000.0)
                    .suffix(" mm")
                    .width(100.0),
            );
        });
        ui.add_space(12.0);

        if ok_cancel(ui, close) || enter_pressed(ui) {
            // Save the choices to preferences.
            app.settings.formats.import_resize_on_import = true;
            app.settings.formats.import_scale = st.scale;
            app.settings.formats.import_width_mm = st.width_mm;
            app.settings.formats.import_height_mm = st.height_mm;
            app.settings.save();
            // Perform the import with the chosen scale.
            let path = st.path.clone();
            let at = None; // Will be set by import_path_placed's import_place setting.
            let scale = st.scale / 100.0;
            app.import_path_placed(&path, at);
            // Apply scaling to the imported shapes.
            if !app.selection.is_empty() {
                app.transform_selection(Affine::scale(scale));
            }
            *close = true;
        }
    });
}
