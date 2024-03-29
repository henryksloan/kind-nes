use nes::NES;

use std::cell::RefCell;
use std::rc::Rc;
use std::thread;
use std::time;

use sdl2::audio::AudioSpecDesired;
use sdl2::controller::{Axis, Button};
use sdl2::event::Event as SDL_Event;
use sdl2::keyboard::Scancode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::WindowCanvas;
use sdl2::video::Window;
use sdl2::Sdl;

const COLORS: &'static [i32] = &[
    0x666666, 0x002A88, 0x1412A7, 0x3B00A4, 0x5C007E, 0x6E0040, 0x6C0600, 0x561D00, 0x333500,
    0x0B4800, 0x005200, 0x004F08, 0x00404D, 0x000000, 0x000000, 0x000000, 0xADADAD, 0x155FD9,
    0x4240FF, 0x7527FE, 0xA01ACC, 0xB71E7B, 0xB53120, 0x994E00, 0x6B6D00, 0x388700, 0x0C9300,
    0x008F32, 0x007C8D, 0x000000, 0x000000, 0x000000, 0xFFFEFF, 0x64B0FF, 0x9290FF, 0xC676FF,
    0xF36AFF, 0xFE6ECC, 0xFE8170, 0xEA9E22, 0xBCBE00, 0x88D800, 0x5CE430, 0x45E082, 0x48CDDE,
    0x4F4F4F, 0x000000, 0x000000, 0xFFFEFF, 0xC0DFFF, 0xD3D2FF, 0xE8C8FF, 0xFBC2FF, 0xFEC4EA,
    0xFECCC5, 0xF7D8A5, 0xE4E594, 0xCFEF96, 0xBDF4AB, 0xB3F3CC, 0xB5EBF2, 0xB8B8B8, 0x000000,
    0x000000,
];

const SAMPLE_RATE: usize = 96000;
const DESIRED_AUDIO_DELAY_MS: usize = 60;
const DELAY_SAMPLES: usize =
    (SAMPLE_RATE as f32 * (DESIRED_AUDIO_DELAY_MS as f32 / 1000.)) as usize;
const AUDIO_BUFF_THRESHOLD: usize = std::mem::size_of::<f32>() * DELAY_SAMPLES;

pub struct SDLUI {
    sdl_context: Sdl,
    canvas: WindowCanvas,
    nes: Rc<RefCell<NES>>,
}

impl SDLUI {
    pub fn new(sdl_context: Sdl, window: Window, nes: Rc<RefCell<NES>>) -> Self {
        Self {
            sdl_context,
            canvas: window.into_canvas().build().unwrap(),
            nes,
        }
    }

    pub fn render_loop(&mut self) {
        let game_controller_subsystem = self.sdl_context.game_controller().unwrap();
        let available = game_controller_subsystem
            .num_joysticks()
            .map_err(|e| format!("can't enumerate joysticks: {}", e))
            .unwrap();
        let controller_opt = (0..available).find_map(|id| {
            if !game_controller_subsystem.is_game_controller(id) {
                return None;
            }
            game_controller_subsystem.open(id).ok()
        });

        self.canvas.set_scale(3.0, 3.0).unwrap();
        self.canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
        let mut event_pump = self.sdl_context.event_pump().unwrap();

        let creator = self.canvas.texture_creator();
        let mut texture = creator
            .create_texture_target(PixelFormatEnum::RGB24, 256, 240)
            .unwrap();

        let mut screen_buff = [0u8; 256 * 240 * 3];

        let audio_subsystem = self.sdl_context.audio().unwrap();
        let desired_spec = AudioSpecDesired {
            freq: Some(SAMPLE_RATE as i32),
            channels: Some(1), // mono
            samples: Some(1024),
        };

        let device = audio_subsystem
            .open_queue::<f32, _>(None, &desired_spec)
            .unwrap();
        device.resume();

        let controls = vec![
            Scancode::Right,
            Scancode::Left,
            Scancode::Down,
            Scancode::Up,
            Scancode::Return,
            Scancode::RShift,
            Scancode::Z,
            Scancode::X,
        ];

        let mut now = time::Instant::now();
        let mut frame_count = 0;
        let frames_per_rate_check = 60;
        let checks_per_rate_report = 2;
        let get_fps = |micros| (1f32 / ((micros / frames_per_rate_check) as f32 * 0.000001)) as u32;

        let mut cycle_interrupt_timer = 1; // Safety check in case controller polling hangs
        let cycles_per_interrupt = 50_000;

        let mut fps_timer = time::Instant::now();
        let mut audio_buff = Vec::new();
        audio_buff.reserve(AUDIO_BUFF_THRESHOLD);
        'main_loop: loop {
            if !self.nes.borrow().has_cartridge() {
                for event in event_pump.poll_iter() {
                    match event {
                        SDL_Event::Quit { .. } => break 'main_loop,
                        _ => {}
                    }
                }
                continue;
            }

            if self.nes.borrow().paused {
                for event in event_pump.poll_iter() {
                    match event {
                        SDL_Event::Quit { .. } => break 'main_loop,
                        _ => {}
                    }
                }

                // Draw the game screen below a gray tint and a pause icon (two parallel lines)
                self.canvas.copy(&texture, None, None).unwrap();
                self.canvas
                    .set_draw_color(sdl2::pixels::Color::RGBA(50, 50, 50, 215));
                let canvas_size = self.canvas.output_size().unwrap();
                self.canvas
                    .fill_rect(sdl2::rect::Rect::new(0, 0, canvas_size.0, canvas_size.1))
                    .unwrap();
                self.canvas
                    .set_draw_color(sdl2::pixels::Color::RGB(225, 25, 25));
                self.canvas
                    .fill_rect(sdl2::rect::Rect::new(
                        ((canvas_size.0 / 6) - 13) as i32,
                        ((canvas_size.1 / 6) - 15) as i32,
                        10,
                        30,
                    ))
                    .unwrap();
                self.canvas
                    .fill_rect(sdl2::rect::Rect::new(
                        ((canvas_size.0 / 6) + 3) as i32,
                        ((canvas_size.1 / 6) - 15) as i32,
                        10,
                        30,
                    ))
                    .unwrap();
                self.canvas.present();
                continue;
            }

            self.nes.borrow_mut().tick();

            if self.nes.borrow().get_shift_strobe() || cycle_interrupt_timer == 0 {
                for event in event_pump.poll_iter() {
                    match event {
                        SDL_Event::Quit { .. } => break 'main_loop,
                        _ => {}
                    }
                }
                const DEAD_ZONE: i16 = 10_000;
                let joy_input = if let Some(controller) = &controller_opt {
                    let joy_x = controller.axis(Axis::LeftX);
                    let joy_y = controller.axis(Axis::LeftY);
                    vec![
                        joy_x > DEAD_ZONE || controller.button(Button::DPadRight), // Right
                        joy_x < -DEAD_ZONE || controller.button(Button::DPadLeft), // Left
                        joy_y > DEAD_ZONE || controller.button(Button::DPadDown),  // Down
                        joy_y < -DEAD_ZONE || controller.button(Button::DPadUp),   // Up
                        controller.button(Button::Start),                          // Start
                        controller.button(Button::Back),                           // Select
                        controller.button(Button::B),                              // B
                        controller.button(Button::A),                              // A
                    ]
                } else {
                    vec![false; 8]
                };

                let mut controller_byte = 0;
                let kb_state = event_pump.keyboard_state();
                for i in 0..controls.len() {
                    let bit = (kb_state.is_scancode_pressed(controls[i]) || joy_input[i]) as u8;
                    controller_byte <<= 1;
                    controller_byte |= bit;
                }
                self.nes
                    .borrow_mut()
                    .try_fill_controller_shift(controller_byte);
            }

            // Basic dynamic sampling idea based on github.com/ltriant/nes:
            // Keep the audio device fed with about DESIRED_AUDIO_DELAY_MS of samples,
            // ceasing sampling while the device is above that threshold.
            let mut buff = self.nes.borrow_mut().take_audio_buff();
            if device.size() < AUDIO_BUFF_THRESHOLD as u32 {
                // Simple volume attenuation, since there's no audio mixing implementation yet
                for entry in buff.iter_mut() {
                    *entry *= 0.25;
                }
                audio_buff.append(&mut buff);
            }

            let framebuffer_option = self.nes.borrow().get_new_frame();
            if let Some(framebuffer) = framebuffer_option {
                device.queue(&audio_buff);
                audio_buff.clear();
                if (frame_count + 1) % frames_per_rate_check == 0 {
                    if (frame_count + 1) % (frames_per_rate_check * checks_per_rate_report) == 0 {
                        self.canvas
                            .window_mut()
                            .set_title(
                                &format!("KindNES | {} fps", get_fps(now.elapsed().as_micros()))[..],
                            )
                            .unwrap();
                    }
                    now = time::Instant::now();
                }
                frame_count += 1;

                let mut pixel_i = 0;
                for y in 0..240 {
                    for x in 0..256 {
                        let color = framebuffer[y][x];
                        let c = COLORS[(color as usize) % 64];
                        let (r, g, b) =
                            ((c >> 16) as u8, ((c >> 8) & 0xFF) as u8, (c & 0xFF) as u8);
                        screen_buff[pixel_i + 0] = r;
                        screen_buff[pixel_i + 1] = g;
                        screen_buff[pixel_i + 2] = b;
                        pixel_i += 3;
                    }
                }
                texture.update(None, &screen_buff, 256 * 3).unwrap();
                self.canvas.copy(&texture, None, None).unwrap();
                self.canvas.present();

                let elapsed = fps_timer.elapsed();
                if elapsed < time::Duration::from_millis(16) {
                    thread::sleep(time::Duration::from_millis(16) - elapsed);
                }
                fps_timer = time::Instant::now();
            }

            cycle_interrupt_timer = (cycle_interrupt_timer + 1) % cycles_per_interrupt;
        }
    }
}
