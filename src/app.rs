#![cfg(target_arch = "wasm32")]
use super::nes::Nes;
use super::ppu::Frame;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use eframe::egui;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

const NES_WIDTH: usize = 256;
const NES_HEIGHT: usize = 240;
const RESOLUTION_SCALING: usize = 3;

const AUDIO_TARGET_SECONDS: f64 = 0.1;
const AUDIO_MAX_SECONDS: f64 = 0.5;
type AudioBuffer = Arc<Mutex<VecDeque<f32>>>;

pub struct NesApp {
    rom_data: Arc<Mutex<Option<Vec<u8>>>>,
    nes: Option<Nes>,
    texture: Option<egui::TextureHandle>,
    message: String,

    audio_buffer: AudioBuffer,
    audio_sample_rate: u32,
    audio_stream: Option<cpal::Stream>,
}

impl NesApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        for theme in [egui::Theme::Dark, egui::Theme::Light] {
            let mut style = (*cc.egui_ctx.style_of(theme)).clone();
            for font_id in style.text_styles.values_mut() {
                font_id.family = egui::FontFamily::Monospace;
            }
            cc.egui_ctx.set_style_of(theme, style);
        }

        Self {
            rom_data: Arc::new(Mutex::new(None)),
            nes: None,
            texture: None,
            message: String::new(),
            audio_buffer: Arc::new(Mutex::new(VecDeque::new())),
            audio_sample_rate: 44100, // will be updated after building Stream
            audio_stream: None,
        }
    }

    fn build_audio_stream(buffer: AudioBuffer) -> Option<(cpal::Stream, u32)> {
        let host = cpal::default_host();
        let device = host.default_output_device()?;
        let config = device.default_output_config().ok()?;
        let sample_rate = config.sample_rate();
        let channels = config.channels() as usize;

        let stream = device
            .build_output_stream(
                config.into(),
                move |data: &mut [f32], _info: &cpal::OutputCallbackInfo| {
                    let mut buf = buffer.lock().unwrap();
                    for frame in data.chunks_mut(channels) {
                        // no-sound if buffer is empty
                        let sample = buf.pop_front().unwrap_or(0.0);
                        for out in frame.iter_mut() {
                            *out = sample;
                        }
                    }
                },
                move |_err| { /* log output if needed */ },
                None,
            )
            .ok()?;
        stream.play().ok()?;
        Some((stream, sample_rate))
    }

    fn open_rom_dialog(&self, ctx: egui::Context) {
        let rom_data_clone = Arc::clone(&self.rom_data);

        wasm_bindgen_futures::spawn_local(async move {
            let file_handle = rfd::AsyncFileDialog::new()
                .add_filter("NES ROM", &["nes"])
                .pick_file()
                .await;

            if let Some(file) = file_handle {
                let rom_bytes = file.read().await;

                if let Ok(mut guard) = rom_data_clone.lock() {
                    *guard = Some(rom_bytes);
                }

                ctx.request_repaint();
            }
        });
    }
}

impl eframe::App for NesApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(mut guard) = self.rom_data.lock()
            && let Some(rom_bytes) = guard.take()
        {
            let result = Nes::new(&rom_bytes);
            match result {
                Ok(nes) => {
                    self.nes = Some(nes);
                }
                Err(_err) => {
                    self.message = String::from("Failed to load ROM");
                }
            }
            self.texture = None;
        }

        if let Some(nes) = &mut self.nes {
            let (key_w, key_a, key_s, key_d, key_j, key_k, key_v, key_b) = ctx.input(|i| {
                (
                    i.key_down(egui::Key::W),
                    i.key_down(egui::Key::A),
                    i.key_down(egui::Key::S),
                    i.key_down(egui::Key::D),
                    i.key_down(egui::Key::J),
                    i.key_down(egui::Key::K),
                    i.key_down(egui::Key::V),
                    i.key_down(egui::Key::B),
                )
            });
            nes.update_button_right(key_d);
            nes.update_button_left(key_a);
            nes.update_button_down(key_s);
            nes.update_button_up(key_w);
            nes.update_button_start(key_b);
            nes.update_button_select(key_v);
            nes.update_button_a(key_k);
            nes.update_button_b(key_j);

            // fill audio buffer and step CPU cycles based on it.
            let target_len = (self.audio_sample_rate as f64 * AUDIO_TARGET_SECONDS) as usize;
            let max_len = (self.audio_sample_rate as f64 * AUDIO_MAX_SECONDS) as usize;
            let current_len = self.audio_buffer.lock().unwrap().len();

            if current_len < target_len {
                let samples_needed = (target_len - current_len).min(max_len);
                let generated_samples = nes.step(samples_needed, self.audio_sample_rate);
                self.audio_buffer.lock().unwrap().extend(generated_samples);
            }

            ctx.request_repaint_after(std::time::Duration::from_secs_f32(1.0 / 60.0));
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        let (key_w, key_a, key_s, key_d, key_j, key_k, key_v, key_b) = ctx.input(|i| {
            (
                i.key_down(egui::Key::W),
                i.key_down(egui::Key::A),
                i.key_down(egui::Key::S),
                i.key_down(egui::Key::D),
                i.key_down(egui::Key::J),
                i.key_down(egui::Key::K),
                i.key_down(egui::Key::V),
                i.key_down(egui::Key::B),
            )
        });

        let mut pressed_keys_text = String::new();
        pressed_keys_text.push_str(if key_w { "W" } else { "-" });
        pressed_keys_text.push_str(if key_a { "A" } else { "-" });
        pressed_keys_text.push_str(if key_s { "S" } else { "-" });
        pressed_keys_text.push_str(if key_d { "D" } else { "-" });
        pressed_keys_text.push_str(if key_j { "J" } else { "-" });
        pressed_keys_text.push_str(if key_k { "K" } else { "-" });
        pressed_keys_text.push_str(if key_v { "V" } else { "-" });
        pressed_keys_text.push_str(if key_b { "B" } else { "-" });

        ui.horizontal(|ui| {
            if ui.button("Reset").clicked()
                && let Some(nes) = &mut self.nes
            {
                nes.reset();
            }

            if ui.button("Open ROM").clicked() {
                // Lazily create the audio stream on the first user interaction.
                // Browsers block audio output until it's started from within a user
                // gesture (e.g. a click), so we can't set this up in `new()`.
                if self.audio_stream.is_none()
                    && let Some((stream, sample_rate)) =
                        Self::build_audio_stream(Arc::clone(&self.audio_buffer))
                {
                    self.audio_sample_rate = sample_rate;
                    self.audio_stream = Some(stream);
                }

                self.open_rom_dialog(ctx.clone());
            }

            if let Some(_nes) = &mut self.nes {
                ui.label("ROM loaded");
            } else {
                ui.label("NoROM(0 bytes)");
            }

            ui.add_space(8.0);
            ui.heading(&pressed_keys_text);
            ui.add_space(8.0);
            ui.heading(&self.message);
        });

        ui.add_space(4.0);

        if let Some(nes) = &mut self.nes {
            let pixels: Vec<egui::Color32> = get_pixels(nes.get_frame());

            let color_image = egui::ColorImage {
                size: [NES_WIDTH, NES_HEIGHT],
                source_size: egui::Vec2 {
                    x: (NES_WIDTH * RESOLUTION_SCALING) as f32,
                    y: (NES_HEIGHT * RESOLUTION_SCALING) as f32,
                },
                pixels,
            };

            let texture_options = egui::TextureOptions::NEAREST;

            if let Some(texture) = &mut self.texture {
                texture.set(color_image, texture_options);
            } else {
                self.texture = Some(ctx.load_texture("nes_screen", color_image, texture_options));
            }
        }

        const RECT_SIZE: egui::Vec2 = egui::Vec2 {
            x: (NES_WIDTH * RESOLUTION_SCALING) as f32,
            y: (NES_HEIGHT * RESOLUTION_SCALING) as f32,
        };

        let available_rect = ui.available_rect_before_wrap();
        let fixed_rect = egui::Rect::from_min_size(
            egui::pos2(available_rect.min.x, available_rect.min.y),
            RECT_SIZE,
        );

        let (_response, painter) = ui.allocate_painter(RECT_SIZE, egui::Sense::hover());

        if let Some(texture) = &self.texture {
            painter.image(
                texture.id(),
                fixed_rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );
        } else {
            painter.rect_filled(fixed_rect, egui::CornerRadius::ZERO, egui::Color32::BLUE);
        }
    }
}

fn get_pixels(frame: &Frame) -> Vec<egui::Color32> {
    let mut pixels: Vec<egui::Color32> = Vec::with_capacity(frame.data.len() / 3);
    for rgb in frame.data.chunks(3) {
        pixels.push(egui::Color32::from_rgb(rgb[0], rgb[1], rgb[2]));
    }
    pixels
}
