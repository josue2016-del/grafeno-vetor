//! Quick text-to-image dialog: a simple text field where the user types a
//! prompt and presses Enter (or clicks Generate) to create a bitmap image
//! on the page. Pre-configures the OpenCode API key if needed.

use crate::app::App;
use crate::i18n::tr;
use crate::ui::dialogs::window;
use egui::{Context, TextEdit};

/// Show the quick AI image generation dialog.
pub fn dialog(app: &mut App, ctx: &Context, close: &mut bool) {
    window(ctx, tr("aigen.quick_generate")).show(ctx, |ui| {
        ui.set_max_width(500.0);
        ui.label(tr("aigen.quick_prompt_label"));
        let mut prompt = app.ai_image.prompt.clone();
        let resp = ui.add(
            TextEdit::multiline(&mut prompt)
                .hint_text(tr("aigen.quick_hint"))
                .desired_rows(3),
        );
        if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) && !ui.input(|i| i.modifiers.shift) {
            // Generate the image.
            if !prompt.trim().is_empty() {
                app.ai_quick_generate(prompt.trim().to_string());
            }
            *close = true;
        }
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            if ui.button(tr("aigen.generate")).clicked() {
                if !prompt.trim().is_empty() {
                    app.ai_quick_generate(prompt.trim().to_string());
                }
                *close = true;
            }
            if ui.button(tr("dialog.cancel")).clicked() {
                *close = true;
            }
        });
    });
}
