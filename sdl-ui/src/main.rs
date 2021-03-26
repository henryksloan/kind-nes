use nes::NES;

use std::cell::RefCell;
use std::env;
use std::fs::File;
use std::process;
use std::rc::Rc;

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

    let nes = Rc::new(RefCell::new(NES::new()));
    nes.borrow_mut().load_rom(file).unwrap_or_else(|err| {
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

    let mut sdl_ui = sdl_ui::SDLUI::new(sdl_context, window, nes);
    sdl_ui.render_loop();
}
