//! Help > About Grafeno Vetor: a blue-to-black banner with the application
//! logo centered, then a short description, the facts (author, license,
//! source code) as a two-column list, the copyright line and a Close button.

use crate::app::App;
use crate::i18n::{tr, trf};
use crate::theme::Tokens;
use crate::ui::brand;
use crate::ui::tips::RichTip;
use egui::{
    Color32, Context, Pos2, Rect, RichText, Sense, Stroke, TextureHandle, Ui, Vec2,
};

/// Banner height in points, including the body margin it covers.
const BANNER_H: f32 = 160.0;
/// The body's top margin, which the banner covers so it meets the title bar.
const BODY_TOP: f32 = 12.0;
/// The body's side margin plus the white edge, which the banner covers so
/// it meets the window border.
const BLEED: f32 = 14.0 + crate::ui::chrome::EDGE;

/// The banner across the top of the dialog body: logo centered on the gradient.
fn banner(ui: &mut Ui, icon: Option<&TextureHandle>) {
    let (slot, _) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), BANNER_H - BODY_TOP),
        Sense::hover(),
    );
    let rect = Rect::from_min_max(
        Pos2::new(slot.left() - BLEED, slot.top() - BODY_TOP),
        Pos2::new(slot.right() + BLEED, slot.bottom()),
    );
    let painter = ui.ctx().layer_painter(ui.layer_id()).with_clip_rect(rect);
    brand::gradient(&painter, rect);

    // Centered logo
    let icon_size = 96.0;
    let icon_rect = Rect::from_center_size(
        rect.center(),
        Vec2::splat(icon_size),
    );
    brand::paint_icon(&painter, icon, icon_rect);
}

/// One row of the facts list: a dim label and its value.
fn fact(ui: &mut Ui, label: &str, value: impl FnOnce(&mut Ui)) {
    ui.label(RichText::new(label).color(Tokens::text_dim()));
    value(ui);
    ui.end_row();
}

/// Draw the About box; sets `close` when it should close.
pub fn about_dialog(app: &mut App, ctx: &Context, close: &mut bool) {
    let icon = brand::icon(ctx);
    crate::ui::chrome::window(tr("dialog.about")).show(ctx, |ui| {
        ui.set_width(480.0);
        ui.spacing_mut().item_spacing.y = 4.0;
        banner(ui, icon.as_ref());
        ui.add_space(16.0);
        ui.add(
            egui::Label::new(
                RichText::new(tr("dialog.about_text"))
                    .size(13.5)
                    .line_height(Some(20.0)),
            )
            .wrap(),
        );
        ui.add_space(12.0);
        let (line, _) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), 1.0), Sense::hover());
        ui.painter().hline(
            line.x_range(),
            line.center().y,
            Stroke::new(1.0, Color32::from_gray(0xD6)),
        );
        ui.add_space(12.0);
        egui::Grid::new("about_facts")
            .num_columns(2)
            .spacing([22.0, 10.0])
            .show(ui, |ui| {
                fact(ui, &tr("dialog.about_author"), |ui| {
                    ui.label(env!("CARGO_PKG_AUTHORS"));
                });
                fact(ui, &tr("dialog.about_license"), |ui| {
                    ui.label(tr("dialog.about_license_value"));
                });
                fact(ui, &tr("dialog.about_source_label"), |ui| {
                    let repo = env!("CARGO_PKG_REPOSITORY");
                    if ui
                        .link(repo)
                        .rich_tip(crate::ui::tips::pair(
                            "dialog.about_source_label",
                            "dialog.about_source_desc",
                        ))
                        .clicked()
                    {
                        app.open_url(repo);
                    }
                });
                fact(ui, &tr("dialog.about_built"), |ui| {
                    ui.add(egui::Label::new(tr("dialog.about_ai")).wrap());
                });
            });
        ui.add_space(14.0);
        ui.label(
            RichText::new(trf(
                "dialog.about_copyright",
                &[("name", env!("CARGO_PKG_AUTHORS"))],
            ))
            .size(12.0)
            .color(Tokens::text_dim()),
        );
        if crate::ui::chrome::button_row(ui, &[&tr("dialog.close")], false).is_some() {
            *close = true;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_about_box_draws_on_the_welcome_screen() {
        let ctx = crate::theme::ui_context();
        let mut app = App::headless();
        app.dialog = crate::ui::dialogs::Dialog::About;
        let mut close = false;
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1600.0, 1000.0),
            )),
            ..Default::default()
        };
        let mut out = ctx.run_ui(input, |ui| {
            about_dialog(&mut app, ui.ctx(), &mut close);
        });
        out.textures_delta.clear();
        assert!(!close);
    }
}
