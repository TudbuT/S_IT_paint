use std::{process, sync::Arc, time::Duration};

use color::{ColorConvert, DrawColor};
use compress::ChangeRect;
use draw::{DrawParams, Location};
use eframe::CreationContext;
use egui::load::SizedTexture;
use egui::*;

use dialog::DialogAction;
use effects::Effects;
use egui_file::FileDialog;
use micro_ndarray::Array;
use mode::Mode;

mod color;
mod compress;
mod dialog;
mod draw;
mod effects;
mod fill;
mod help;
mod io;
mod mode;
mod pull;
mod tex;

fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Paint",
        native_options,
        Box::new(|cc| Box::new(App::new(cc))),
    )
    .unwrap();
}

pub struct App {
    pub image: Array<Color32, 2>,
    pub real_image: Array<Color32, 2>,
    pub changes: ChangeRect,
    pub tex: TextureId,

    pub filename: Option<String>,
    pub dialog_action: Option<DialogAction>,
    pub dialog: Option<FileDialog>,

    pub color: DrawColor,
    pub draw: DrawParams,
    pub last_mouse_pos: Option<DrawParams>,

    pub mode: Mode,
    pub effects: Effects,

    pub pull_start: Option<[usize; 2]>,
    pub eraser: bool,

    pub(crate) cur_edit: Option<String>,
    pub(crate) pixels_per_point: f32,
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
            effects: Effects::default(),
            pull_start: None,
            eraser: false,
            pixels_per_point: 1.0,
        }
    }

    pub fn to_px(&self, point: f32) -> usize {
        (point * self.pixels_per_point) as usize
    }
    pub fn to_point(&self, px: usize) -> f32 {
        px as f32 / self.pixels_per_point
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // try to do 90fps
        ctx.request_repaint_after(Duration::from_millis(1000 / 90));
        self.pixels_per_point = ctx.pixels_per_point();

        self.handle_dialogs(ctx);

        // the content frame
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
                    ui.menu_button("Effects", |ui| {
                        ui.checkbox(&mut self.effects.randomize_size, "Randomize sizes");
                        ui.checkbox(&mut self.effects.checkerboard, "Checkerboard");
                    });
                    ui.menu_button("Help", |ui| self.render_help(ui));
                    ui.add_space(50.0);
                    ui.checkbox(&mut self.eraser, "Eraser");
                })
            })
        });

        // updates things set in the debug menu
        self.update_effects();

        CentralPanel::default().frame(f).show(ctx, |ui| {
            align_cursor(ui);
            // shows the image
            let size = ui.available_size().floor();
            self.correct_tex_size(
                &mut ctx.tex_manager().write(),
                [self.to_px(size.x), self.to_px(size.y)],
            );
            self.image_to_texture(&mut ctx.tex_manager().write());

            // draw the texture
            let r = ui.add(
                Image::from_texture(SizedTexture::new(self.tex, size)).fit_to_original_size(1.0),
            );

            // handle keyboard and mouse input
            ui.input(|inp| {
                // get pointer pos offset to be in the image or return if its not inside the window
                let Some(pointer_pos) = r.hover_pos().map(|pos| pos - r.rect.min) else {
                    return;
                };
                let pos = Location::new(self.to_px(pointer_pos.x), self.to_px(pointer_pos.y));
                // return if not actually on the image
                if !r.hovered() {
                    return; // we don't need to handle it if it's not in focus
                }

                // handle eraser
                if self.eraser {
                    if inp.pointer.primary_down() {
                        self.draw_mouse(
                            DrawParams {
                                loc: pos,
                                size: 20,
                                px: 0xffffff,
                            },
                            App::draw_dot,
                        );
                    }
                    return;
                }

                // handle pulling shapes
                if self.pull_start.is_some() || inp.pointer.secondary_down() {
                    self.pull(inp, [pos.x, pos.y]);
                    return;
                }

                if inp.key_down(Key::D) {
                    self.draw_ngon(
                        self.draw.at_loc(pos),
                        3,
                        self.draw.size.max(1) as f32 * 30.0,
                        self.draw.size.max(1) as f32 * 30.0,
                        0.0,
                    );
                }
                if inp.key_down(Key::Q) {
                    self.draw_ngon(
                        self.draw.at_loc(pos),
                        4,
                        self.draw.size.max(1) as f32 * 30.0,
                        self.draw.size.max(1) as f32 * 30.0,
                        45.0,
                    );
                }
                if inp.key_down(Key::K) {
                    self.draw_ngon(
                        self.draw.at_loc(pos),
                        0,
                        self.draw.size.max(1) as f32 * 30.0,
                        self.draw.size.max(1) as f32 * 30.0,
                        0.0,
                    );
                }
                // a normal draw operation. interpolates unless the operation overrides it
                if inp.pointer.primary_down() {
                    let draw = self.draw.at_loc(pos);
                    if self.mode.run_once() {
                        // don't interpolate
                        self.mode.into_fn()(self, draw);
                    } else {
                        // interpolate
                        self.draw_mouse(draw, self.mode.into_fn());
                    }
                } else {
                    self.last_mouse_pos = None;
                }
            });
        });
    }
}

// cursor may be between pixels, ew!
fn align_cursor(ui: &mut Ui) {
    let cursor = ui.cursor();
    let diff = cursor.min.ceil() - cursor.min;
    ui.allocate_space(vec2(diff.x, diff.y));
}
