use egui::*;

use crate::App;

pub enum DialogAction {
    Open,
    Save,
}

impl App {
    pub fn open_file(&mut self) {
        let mut dialog = egui_file::FileDialog::open_file(None);
        dialog.open();
        self.dialog_action = Some(DialogAction::Open);
        self.dialog = Some(dialog);
    }

    pub fn save_file(&mut self, ask_name: bool) {
        if ask_name || self.filename.is_none() {
            let mut dialog = egui_file::FileDialog::save_file(None);
            dialog.open();
            self.dialog_action = Some(DialogAction::Save);
            self.dialog = Some(dialog);
        } else {
            self.save();
        }
    }

    pub fn handle_dialogs(&mut self, ctx: &Context) {
        if let Some(ref mut d) = self.dialog {
            if d.show(ctx).selected() {
                if let Some(file) = d.path() {
                    match self.dialog_action.as_ref().unwrap() {
                        DialogAction::Open => {
                            self.filename =
                                Some(file.to_str().expect("invalid file name").to_owned());
                            self.load();
                        }
                        DialogAction::Save => {
                            self.filename =
                                Some(file.to_str().expect("invalid file name").to_owned());
                            self.save();
                        }
                    };
                    self.dialog_action = None;
                    self.dialog = None;
                }
            }
        }
    }
}
