//! File > Save to Google Drive: a stub dialog that asks the user to sign in
//! in their browser, then prepares to save the drawing to Google Drive.
//!
//! This is a basic structure; full Drive API integration is planned for later.

use crate::app::App;
use crate::i18n::{tr, trf};
use crate::ui::dialogs::window;
use egui::{Context, RichText};

/// Holds the state of the Google Drive save dialog.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct GoogleDriveState {
    /// The file name to use when saving.
    pub file_name: String,
    /// The folder ID or path in Google Drive.
    pub folder: String,
    /// Status message (e.g. "Signed in as ...", errors).
    pub status: String,
    /// Whether we are currently signed in.
    pub signed_in: bool,
    /// The email of the signed-in user (if any).
    pub email: String,
}

/// URL for Google Drive.
const GOOGLE_DRIVE_URL: &str = "https://drive.google.com";

impl App {
    /// Returns the initial Google Drive save state, prefilled with the
    /// current document's title.
    pub fn google_drive_state(&self) -> GoogleDriveState {
        GoogleDriveState {
            file_name: self.document_title().trim_end_matches('*').to_string(),
            folder: String::new(),
            status: String::new(),
            signed_in: false,
            email: String::new(),
        }
    }

    /// Opens Google Drive in the user's browser.
    pub fn google_drive_open_auth(&mut self) {
        self.open_url(GOOGLE_DRIVE_URL);
        self.google_drive.status = tr("google.drive_opened");
    }
}

/// Draw the Google Drive save dialog.
pub fn google_drive_dialog(app: &mut App, ctx: &Context, st: &mut GoogleDriveState, close: &mut bool) {
    window(ctx, tr("google.drive_title")).show(ctx, |ui| {
        ui.set_max_width(420.0);
        if st.signed_in {
            ui.label(RichText::new(trf("google.drive_signed_in_as", &[("email", &st.email)])).strong());
        } else {
            ui.label(tr("google.drive_login"));
            if ui
                .add(
                    egui::Button::new(tr("google.login_button"))
                        .fill(egui::Color32::from_rgb(0x1A, 0x73, 0xE8)),
                )
                .on_hover_text(tr("google.login_tip"))
                .clicked()
            {
                app.google_drive_open_auth();
            }
        }
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label(tr("google.drive_file_name"));
            ui.add(egui::TextEdit::singleline(&mut st.file_name).desired_width(250.0));
        });
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label(tr("google.drive_folder"));
            ui.add(egui::TextEdit::singleline(&mut st.folder).desired_width(250.0));
        });
        ui.add_space(8.0);
        if ui.button(tr("google.drive_save")).clicked() {
            #[cfg(not(target_arch = "wasm32"))]
            {
                use crate::export::export;
                use crate::ui::dialogs::ExportState;
                let name = st.file_name.clone().trim_end_matches('.').to_string();
                let path = std::env::temp_dir().join(format!("{}-drive.pdf", name));
                let st_export = ExportState::default();
                match export(app, &path, &st_export) {
                    Ok(_) => {
                        app.google_drive_open_auth();
                        st.status = trf("google.drive_saved_path", &[
                            ("path", &path.display().to_string()),
                            ("name", &name),
                        ]);
                    }
                    Err(e) => {
                        st.status = e;
                    }
                }
            }
            #[cfg(target_arch = "wasm32")]
            {
                app.google_drive_open_auth();
                st.status = tr("google.drive_opened");
            }
        }
        if !app.google_drive.status.is_empty() {
            ui.add_space(8.0);
            ui.label(egui::RichText::new(&app.google_drive.status).color(egui::Color32::from_rgb(0x42, 0x85, 0xF4)));
        }
        if crate::ui::chrome::button_row(ui, &[&tr("dialog.close")], false).is_some() {
            *close = true;
        }
    });
}
