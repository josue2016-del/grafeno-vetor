//! The Missing Fonts dialog (#936): after a drawing opens with fonts that
//! are not installed, a row per missing family shows the installed font
//! drawing its text (the automatic choice). The dialog is informational in
//! this version: it lists each missing family and the installed font that
//! draws its text, and lets the user dismiss it for this session or stop
//! asking (Options > Global > Warnings). See docs/behavior/missing-fonts.md.

use crate::app::App;
use crate::i18n::tr;
use crate::ui::dialogs::window;
use egui::Context;

/// One missing family.
#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    pub missing: String,
    /// The installed font drawing its text.
    pub substitute: Option<String>,
}

/// The dialog's state.
#[derive(Debug, Clone, PartialEq)]
pub struct MissingFontsState {
    /// The drawing's file name.
    pub name: String,
    /// Objects of the file that could not be read, said after this dialog.
    pub unread: usize,
    pub rows: Vec<Row>,
    /// Do not ask again (Options > Global > Warnings).
    pub dont_show: bool,
    /// The installed families typed into the list's filter.
    pub filter: String,
}

impl MissingFontsState {
    /// A row per missing family, each with the font drawing it now.
    pub fn new(name: String, unread: usize, missing: &[String]) -> Self {
        let fonts = grafeno_text::fonts();
        let rows = missing
            .iter()
            .map(|m| Row {
                missing: m.clone(),
                substitute: fonts.substitute(m),
            })
            .collect();
        MissingFontsState {
            name,
            unread,
            rows,
            dont_show: false,
            filter: String::new(),
        }
    }
}

impl App {
    /// OK: dismiss the dialog; optionally stop asking about missing fonts.
    pub fn apply_font_choices(&mut self, st: &MissingFontsState) {
        if st.dont_show {
            self.settings.warnings.missing_fonts = false;
            self.settings.save();
        }
        // Re-check: the status line shows the current stand-ins.
        self.check_missing_fonts();
    }
}

pub fn dialog(app: &mut App, ctx: &Context, st: &mut MissingFontsState, close: &mut bool) {
    let mut help = false;
    let mut ok = false;
    window(ctx, tr("missing_fonts.title")).show(ctx, |ui| {
        ui.set_width(520.0);
        ui.label(crate::i18n::trf(
            "missing_fonts.intro",
            &[("name", &st.name)],
        ));
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.label(tr("missing_fonts.filter"));
            ui.add(egui::TextEdit::singleline(&mut st.filter).desired_width(200.0));
        });
        ui.add_space(4.0);
        let filter = st.filter.trim().to_lowercase();
        let families = &app.font_families;
        egui::ScrollArea::vertical()
            .max_height(260.0)
            .show(ui, |ui| {
                egui::Grid::new("missing_fonts_grid")
                    .num_columns(2)
                    .spacing([16.0, 6.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.strong(tr("missing_fonts.missing"));
                        ui.strong(tr("missing_fonts.substitute"));
                        ui.end_row();
                        for row in st.rows.iter_mut() {
                            if !filter.is_empty()
                                && !row.missing.to_lowercase().contains(&filter)
                            {
                                continue;
                            }
                            ui.label(&row.missing);
                            let shown = row
                                .substitute
                                .as_ref()
                                .map(|s| s.clone())
                                .unwrap_or_else(|| tr("missing_fonts.none").to_string());
                            let _ = families;
                            ui.label(shown);
                            ui.end_row();
                        }
                    });
            });
        ui.add_space(6.0);
        ui.checkbox(&mut st.dont_show, tr("missing_fonts.dont_ask"));
        ui.add_space(4.0);
        let (o, h) = crate::ui::dialogs::help_ok_cancel(ui, close);
        ok = o;
        help = h;
    });
    if ok {
        app.apply_font_choices(st);
    }
    if help {
        app.open_dialog_help();
    }
    if *close && st.unread > 0 {
        // The note about the objects that could not be read follows.
        app.dialog = crate::ui::dialogs::Dialog::OpenNotes(crate::ui::dialogs::OpenNotes {
            name: st.name.clone(),
            unread: st.unread,
            fonts: Vec::new(),
            dont_show: false,
        });
    }
}
