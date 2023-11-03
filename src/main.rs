use std::f32::consts::PI;
use std::{process, sync::Arc, time::Duration};

use color::DrawColor;
use compress::ChangeRect;
use eframe::CreationContext;
use egui::load::SizedTexture;
use egui::*;

use dialog::DialogAction;
use egui_file::FileDialog;
use micro_ndarray::Array;
use mode::Mode;

mod color;
mod compress;
mod dialog;
mod draw;
mod io;
mod mode;
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
    pub tex: TextureId,
    pub filename: Option<String>,
    pub dialog_action: Option<DialogAction>,
    pub dialog: Option<FileDialog>,
    pub last_mouse_pos: Option<[usize; 2]>,
    pub mode: Mode,
    pub color: DrawColor,
    pub changes: ChangeRect,
}

impl App {
    pub fn new(cc: &CreationContext) -> App {
        Self {
            image: Array::new_with([100, 100], Color32::WHITE),
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
            changes: ChangeRect::new(20),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_millis(1000 / 60));
        self.handle_dialogs(ctx);
        let f = Frame::none()
            .inner_margin(Margin::same(2.0))
            .fill(Color32::from_rgb(0x00, 0x00, 0x00));
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
                    ui.menu_button("Colors", |ui| {
                        DrawColor::menu(self, ui);
                    });
                })
            })
        });
        CentralPanel::default().frame(f).show(ctx, |ui| {
            let size = ui.available_size();
            self.correct_tex_size(
                &mut ctx.tex_manager().write(),
                [size.x as usize, size.y as usize],
            );
            self.image_to_texture(&mut ctx.tex_manager().write());
            let r = ui.add(egui::Image::from_texture(SizedTexture::new(self.tex, size)));
            ui.input(|inp| {
                if !r.hovered() {
                    return;
                }
                let Some(pointer_pos) = r.hover_pos().map(|x| x - r.rect.min) else {
                    return;
                };
                if inp.key_down(Key::D) {
                    self.draw_ngon(pointer_pos.x as usize, pointer_pos.y as usize, 3, 30.0, 0.0);
                }
                if inp.key_down(Key::Q) {
                    self.draw_ngon(
                        pointer_pos.x as usize,
                        pointer_pos.y as usize,
                        4,
                        30.0,
                        45.0,
                    );
                }
                if inp.key_down(Key::K) {
                    self.draw_ngon(
                        pointer_pos.x as usize,
                        pointer_pos.y as usize,
                        (30.0 * PI) as usize,
                        30.0,
                        0.0,
                    );
                }
                if inp.pointer.primary_down() {
                    self.draw_mouse(
                        pointer_pos.x as usize,
                        pointer_pos.y as usize,
                        self.mode.into_fn(),
                    );
                } else {
                    self.last_mouse_pos = None;
                }
            });
        });
    }
}
