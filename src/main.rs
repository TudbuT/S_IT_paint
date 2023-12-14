use std::{process, sync::Arc, time::Duration};

use color::{ColorConvert, DrawColor};
use compress::ChangeRect;
use draw::DrawParams;
use eframe::CreationContext;
use egui::load::SizedTexture;
use egui::*;

use debug::Debug;
use dialog::DialogAction;
use egui_file::FileDialog;
use micro_ndarray::Array;
use mode::Mode;

mod color;
mod compress;
mod debug;
mod dialog;
mod draw;
mod fill;
mod io;
mod mode;
mod pull;
mod tex;

fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Zeichenprogramm",
        native_options,
        Box::new(|cc| Box::new(App::new(cc))),
    )
    .unwrap();
}

pub struct App {
    pub image: Array<Color32, 2>,
    pub real_image: Array<Color32, 2>,
    pub tex: TextureId,
    pub filename: Option<String>,
    pub dialog_action: Option<DialogAction>,
    pub dialog: Option<FileDialog>,
    pub last_mouse_pos: Option<DrawParams>,
    pub mode: Mode,
    pub color: DrawColor,
    pub draw: DrawParams,
    pub changes: ChangeRect,
    pub cur_edit: Option<String>,
    pub debug: Debug,
    pub pull_start: Option<[usize; 2]>,
}

impl App {
    pub fn new(cc: &CreationContext) -> App {
        Self {
            image: Array::new_with([100, 100], Color32::WHITE),
            real_image: Array::new_with([100, 100], Color32::WHITE),
            tex: cc.egui_ctx.tex_manager().write().alloc(
                "canvas".to_owned(),
                ImageData::Color(Arc::new(ColorImage::new(
                    [100, 100],
                    Color32::TEMPORARY_COLOR,
                ))),
                TextureOptions {
                    magnification: TextureFilter::Nearest,
                    minification: TextureFilter::Linear,
                },
            ),
            filename: None,
            dialog_action: None,
            dialog: None,
            last_mouse_pos: None,
            mode: Mode::Paintbrush,
            color: DrawColor::Black,
            draw: DrawParams::new(0, 0, 1, 0x000000),
            changes: ChangeRect::new(20),
            cur_edit: None,
            debug: Debug::default(),
            pull_start: None,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_millis(0));
        self.handle_dialogs(ctx);
        let f = Frame::none()
            .inner_margin(Margin::same(2.0))
            .fill(Color32::from_rgb(0x00, 0x00, 0x00));

        // create the menu bar
        TopBottomPanel::top("menubar").frame(f).show(ctx, |ui| {
            menu::bar(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.menu_button("File", |ui| {
                        if self.filename.is_some() && ui.button("Reload from file").clicked() {
                            self.load();
                        }
                        if ui.button("Clear").clicked() {
                            // expanded automatically
                            self.image = Array::new_with([0, 0], Color32::WHITE);
                        }
                        if ui.button("Open...").clicked() {
                            self.open_file();
                        }
                        if ui.button("Save").clicked() {
                            self.save_file(false);
                        }
                        if ui.button("Save as...").clicked() {
                            self.save_file(true);
                        }
                        if ui.button("Close").clicked() {
                            process::exit(0);
                        }
                    });
                    ui.menu_button("Tools", |ui| {
                        Mode::menu(self, ui);
                    });
                    ui.menu_button("Color", |ui| {
                        DrawColor::menu(self, ui);
                    });
                    let mut col = self.draw.px.into_colorf();
                    if ui.color_edit_button_rgb(&mut col).changed() {
                        self.draw.px = col.into_color();
                    }
                    ui.menu_button("Size", |ui| {
                        if self.cur_edit.is_none() {
                            self.cur_edit = Some(self.draw.size.to_string());
                        }
                        let cur_edit = self.cur_edit.as_mut().unwrap();
                        if ui.text_edit_singleline(cur_edit).lost_focus() {
                            self.draw.size = cur_edit.parse().unwrap_or(self.draw.size);
                            self.cur_edit = None;
                        }
                    });
                    ui.menu_button("Debugging", |ui| {
                        ui.checkbox(&mut self.debug.randomize_size, "Randomize sizes");
                    });
                })
            })
        });

        // updates things set in the debug menu
        self.update_debug();

        CentralPanel::default().frame(f).show(ctx, |ui| {
            // shows the image
            let size = ui.available_size().round();
            self.correct_tex_size(
                &mut ctx.tex_manager().write(),
                [size.x as usize, size.y as usize],
            );
            self.image_to_texture(&mut ctx.tex_manager().write());
            let r = ui.add(Image::from_texture(SizedTexture::new(self.tex, size)));

            // handle mouse and keyboard input
            ui.input(|inp| {
                if !r.hovered() {
                    return; // we don't need to handle it if it's not in focus
                }
                let Some(pointer_pos) = r.hover_pos().map(|x| x - r.rect.min) else {
                    return; // we don't need to handle it if the image is not under the cursor
                };
                if inp.pointer.secondary_down() || self.pull_start.is_some() {
                    self.pull(&inp, [pointer_pos.x as usize, pointer_pos.y as usize]);
                    return;
                }
                if inp.key_down(Key::D) {
                    self.draw_ngon(
                        self.draw.at(pointer_pos.x as usize, pointer_pos.y as usize),
                        3,
                        self.draw.size.max(1) as f32 * 30.0,
                        self.draw.size.max(1) as f32 * 30.0,
                        0.0,
                    );
                }
                if inp.key_down(Key::Q) {
                    self.draw_ngon(
                        self.draw.at(pointer_pos.x as usize, pointer_pos.y as usize),
                        4,
                        self.draw.size.max(1) as f32 * 30.0,
                        self.draw.size.max(1) as f32 * 30.0,
                        45.0,
                    );
                }
                if inp.key_down(Key::K) {
                    self.draw_ngon(
                        self.draw.at(pointer_pos.x as usize, pointer_pos.y as usize),
                        0,
                        self.draw.size.max(1) as f32 * 30.0,
                        self.draw.size.max(1) as f32 * 30.0,
                        0.0,
                    );
                }
                // a normal draw operation. interpolates unless the operation overrides it
                if inp.pointer.primary_down() {
                    let draw = self.draw.at(pointer_pos.x as usize, pointer_pos.y as usize);
                    if self.mode.run_once() {
                        self.mode.into_fn()(self, draw);
                    } else {
                        self.draw_mouse(draw, self.mode.into_fn());
                    }
                } else {
                    self.last_mouse_pos = None;
                }
            });
        });
    }
}
