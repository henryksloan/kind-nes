#![windows_subsystem = "windows"]

use nes::NES;
use sdl_frontend::SDLFrontend;

use core::ffi::c_void;
use std::cell::RefCell;
use std::env;
use std::fs::File;
use std::rc::Rc;

use sdl2::video::Window as SDL_Window;
use sdl2_sys::SDL_CreateWindowFrom;

extern crate native_windows_derive as nwd;
extern crate native_windows_gui as nwg;

use nwd::NwgUi;
use nwg::NativeUi;

#[derive(Default, NwgUi)]
pub struct BasicApp {
    nes: Rc<RefCell<NES>>,

    #[nwg_control(size: (256 * 3, 240 * 3), position: (300, 300), title: "KindNES", flags: "WINDOW|VISIBLE")]
    window: nwg::Window,

    #[nwg_control(text: "File")]
    file_menu: nwg::Menu,

    #[nwg_control(text: "Open ROM", parent: file_menu)]
    #[nwg_events( OnMenuItemSelected: [BasicApp::open_rom_dialog] )]
    open_item: nwg::MenuItem,

    #[nwg_control(parent: file_menu)]
    exit_separator: nwg::MenuSeparator,

    #[nwg_control(text: "Exit", parent: file_menu)]
    #[nwg_events( OnMenuItemSelected: [BasicApp::exit] )]
    exit_item: nwg::MenuItem,

    #[nwg_resource(action: FileDialogAction::Open, title: "Open a .NES file")]
    file_dialog: nwg::FileDialog,
}

impl BasicApp {
    fn open_rom_dialog(&self) {
        if self.file_dialog.run(Some(&self.window)) {
            if let Ok(item) = self.file_dialog.get_selected_item() {
                match File::open(&item) {
                    Ok(file) => self.nes.borrow_mut().load_rom(file).unwrap_or_else(|err| {
                        nwg::modal_error_message(
                            &self.window,
                            "Error loading ROM",
                            &format!(
                                "There was an error when loading the ROM in {:?}: {:?}",
                                item, err
                            ),
                        );
                    }),
                    Err(err) => {
                        nwg::modal_error_message(
                            &self.window,
                            "Error loading file",
                            &format!("There was an error when loading {:?}: {:?}", item, err),
                        );
                    }
                }
            }
        }
    }

    fn exit(&self) {
        nwg::stop_thread_dispatch();
        std::process::exit(0);
    }
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");
    let app = BasicApp::build_ui(Default::default()).expect("Failed to build UI");

    let [total_width, total_height] = [nwg::Monitor::width(), nwg::Monitor::height()];
    let (width, height) = (256 * 3, 240 * 3);
    let x = (total_width - width) / 2;
    let y = (total_height - height) / 2;

    app.window.set_position(x, y);

    let args: Vec<String> = env::args().collect();
    if args.len() >= 2 {
        match File::open(&args[1]) {
            Ok(file) => app.nes.borrow_mut().load_rom(file).unwrap_or_else(|err| {
                println!("failed to load ROM: {}", err);
            }),
            Err(err) => println!("failed to read file: {}", err),
        }
    }

    let window = unsafe {
        let window_raw =
            SDL_CreateWindowFrom(app.window.handle.hwnd().unwrap() as *mut _ as *mut c_void);
        SDL_Window::from_ll(video_subsystem, window_raw)
    };

    let mut sdl_frontend = SDLFrontend::new(sdl_context, window, app.nes.clone());
    sdl_frontend.render_loop();
}
