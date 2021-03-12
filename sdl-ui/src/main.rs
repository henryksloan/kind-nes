use nes;
use nes::NES;

use std::env;
use std::fs::File;
use std::process;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;

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

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: {} <NES ROM file>", args[0]);
        process::exit(1);
    }

    let file = File::open(&args[1]).unwrap_or_else(|err| {
        println!("failed to read file: {}", err);
        process::exit(1);
    });

    let mut nes = NES::new();
    nes.load_rom(file).unwrap_or_else(|err| {
        println!("failed to load ROM: {}", err);
        process::exit(1);
    });

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("KindNES", (256.0 * 3.0) as u32, (240.0 * 3.0) as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    canvas.set_scale(3.0, 3.0).unwrap();

    let creator = canvas.texture_creator();
    let mut texture = creator
        .create_texture_target(PixelFormatEnum::RGB24, 256, 240)
        .unwrap();

    let mut screen_buff = [0u8; 256 * 240 * 3];

    use std::time::Instant;
    let mut now = Instant::now();
    let mut frame_count = 0;
    let frames_per_rate_check = 60;
    let checks_per_rate_report = 2;
    let get_fps = |micros| (1f32 / ((micros / frames_per_rate_check) as f32 * 0.000001)) as u32;
    loop {
        nes.tick();

        if frame_count % 10 == 0 { // Temporary fix for slow event poll
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => std::process::exit(0),
                    _ => {}
                }
            }
        }

        if let Some(framebuffer) = nes.get_new_frame() {
            if (frame_count + 1) % frames_per_rate_check == 0 {
                if (frame_count + 1) % (frames_per_rate_check * checks_per_rate_report) == 0 {
                    canvas
                        .window_mut()
                        .set_title(
                            &format!("KindNES | {} fps", get_fps(now.elapsed().as_micros()))[..],
                        )
                        .unwrap();
                }
                now = Instant::now();
            }
            frame_count += 1;

            let mut pixel_i = 0;
            let mut update = false;
            for y in 0..240 {
                for x in 0..256 {
                    let color = framebuffer[y][x];
                    let c = COLORS[(color as usize) % 64];
                    let (r, g, b) = ((c >> 16) as u8, ((c >> 8) & 0xFF) as u8, (c & 0xFF) as u8);
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
                canvas.copy(&texture, None, None).unwrap();
                canvas.present();
            }
        }
    }
}
