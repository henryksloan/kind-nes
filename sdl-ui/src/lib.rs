use nes::NES;

use std::cell::RefCell;
use std::rc::Rc;
use std::thread;
use std::time;

use sdl2::audio::AudioSpecDesired;
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
        self.canvas.set_scale(3.0, 3.0).unwrap();
        let mut event_pump = self.sdl_context.event_pump().unwrap();

        let creator = self.canvas.texture_creator();
        let mut texture = creator
            .create_texture_target(PixelFormatEnum::RGB24, 256, 240)
            .unwrap();

        let mut screen_buff = [0u8; 256 * 240 * 3];

        let audio_subsystem = self.sdl_context.audio().unwrap();
        let desired_spec = AudioSpecDesired {
            freq: Some(96000),
            channels: Some(1), // mono
            samples: None,     // default sample size
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
        loop {
            if !self.nes.borrow().has_cartridge() {
                for event in event_pump.poll_iter() {
                    match event {
                        SDL_Event::Quit { .. } => std::process::exit(0),
                        _ => {}
                    }
                }
                continue;
            }

            self.nes.borrow_mut().tick();

            if self.nes.borrow().get_shift_strobe() || cycle_interrupt_timer == 0 {
                for event in event_pump.poll_iter() {
                    match event {
                        SDL_Event::Quit { .. } => std::process::exit(0),
                        _ => {}
                    }
                }

                let mut controller_byte = 0;
                let kb_state = event_pump.keyboard_state();
                for scancode in &controls {
                    let bit = kb_state.is_scancode_pressed(*scancode) as u8;
                    controller_byte <<= 1;
                    controller_byte |= bit;
                }
                self.nes
                    .borrow_mut()
                    .try_fill_controller_shift(controller_byte);
            }

            let framebuffer_option = self.nes.borrow().get_new_frame();
            if let Some(framebuffer) = framebuffer_option {
                device.queue(&self.nes.borrow_mut().take_audio_buff());
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
                let mut update = false;
                for y in 0..240 {
                    for x in 0..256 {
                        let color = framebuffer[y][x];
                        let c = COLORS[(color as usize) % 64];
                        let (r, g, b) =
                            ((c >> 16) as u8, ((c >> 8) & 0xFF) as u8, (c & 0xFF) as u8);
                        if screen_buff[pixel_i + 0] != r
                            || screen_buff[pixel_i + 1] != g
                            || screen_buff[pixel_i + 2] != b
                        {
                            screen_buff[pixel_i + 0] = r;
                            screen_buff[pixel_i + 1] = g;
                            screen_buff[pixel_i + 2] = b;
                            update = true;
                        }
                        pixel_i += 3;
                    }
                }
                if update {
                    texture.update(None, &screen_buff, 256 * 3).unwrap();
                    self.canvas.copy(&texture, None, None).unwrap();
                    self.canvas.present();
                }

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
