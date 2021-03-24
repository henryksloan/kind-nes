//! # System MenuBar Sample
//!
//! This sample demonstrates how to create a "system" menu bar. It should always be preferred
//! over the `gtk::MenuBar` since it adapts to the targetted system.

extern crate gio;
extern crate glib;
extern crate gtk;

use gdk::Window as GdkWindow;
use gio::prelude::*;
use glib::clone;
use gtk::prelude::*;
use gtk::AboutDialog;
use gtk::DrawingArea;
use gtk::DrawingAreaBuilder;
use gtk_sys::gtk_widget_set_size_request;
use std::fs::File;

use sdl2::video::Window as SDL_Window;
use sdl2_sys::{SDL_CreateWindowFrom, SDL_Quit};

use core::ffi::c_void;
use std::cell::RefCell;
use std::env::args;
use std::rc::Rc;

use nes::NES;
use sdl_frontend::SDLFrontend;

extern "C" {
    fn gdk_win32_window_get_handle(window: GdkWindow) -> *mut c_void;
}

fn build_system_menu(application: &gtk::Application) {
    let menu = gio::Menu::new();
    let menu_bar = gio::Menu::new();
    let more_menu = gio::Menu::new();
    let switch_menu = gio::Menu::new();
    let settings_menu = gio::Menu::new();
    let submenu = gio::Menu::new();

    // The first argument is the label of the menu item whereas the second is the action name. It'll
    // makes more sense when you'll be reading the "add_actions" function.
    menu.append(Some("Quit"), Some("app.quit"));

    switch_menu.append(Some("Switch"), Some("app.switch"));
    menu_bar.append_submenu(Some("_Switch"), &switch_menu);

    settings_menu.append(Some("Sub another"), Some("app.sub_another"));
    submenu.append(Some("Sub sub another"), Some("app.sub_sub_another"));
    submenu.append(Some("Sub sub another2"), Some("app.sub_sub_another2"));
    settings_menu.append_submenu(Some("Sub menu"), &submenu);
    menu_bar.append_submenu(Some("_Another"), &settings_menu);

    more_menu.append(Some("About"), Some("app.about"));
    menu_bar.append_submenu(Some("?"), &more_menu);

    application.set_app_menu(Some(&menu));
    application.set_menubar(Some(&menu_bar));
}

/// This function creates "actions" which connect on the declared actions from the menu items.
fn add_actions(
    application: &gtk::Application,
    switch: &gtk::Switch,
    label: &gtk::Label,
    drawing_area: &gtk::DrawingArea,
    window: &gtk::ApplicationWindow,
) {
    // Thanks to this method, we can say that this item is actually a checkbox.
    let switch_action = gio::SimpleAction::new_stateful("switch", None, &false.to_variant());
    switch_action.connect_activate(clone!(@weak switch => move |g, _| {
        let mut is_active = false;
        if let Some(g) = g.get_state() {
            is_active = g.get().expect("couldn't get bool");
            // We update the state of the toggle.
            switch.set_active(!is_active);
        }
        // We need to change the toggle state ourselves. `gio` dark magic.
        g.change_state(&(!is_active).to_variant());
    }));

    // The same goes the around way: if we update the switch state, we need to update the menu
    // item's state.
    switch.connect_property_active_notify(clone!(@weak switch_action => move |s| {
        switch_action.change_state(&s.get_active().to_variant());
    }));

    let sub_another = gio::SimpleAction::new("sub_another", None);
    sub_another.connect_activate(clone!(@weak label => move |_, _| {
        label.set_text("sub another menu item clicked");
    }));
    let sub_sub_another = gio::SimpleAction::new("sub_sub_another", None);
    sub_sub_another.connect_activate(clone!(@weak label => move |_, _| {
        label.set_text("sub sub another menu item clicked");
    }));
    let sub_sub_another2 = gio::SimpleAction::new("sub_sub_another2", None);
    sub_sub_another2.connect_activate(clone!(@weak label => move |_, _| {
        label.set_text("sub sub another2 menu item clicked");
    }));

    let quit = gio::SimpleAction::new("quit", None);
    quit.connect_activate(clone!(@weak window => move |_, _| {
        window.close();
    }));

    let about = gio::SimpleAction::new("about", None);
    about.connect_activate(clone!(@weak window => move |_, _| {
        let p = AboutDialog::new();
        p.set_website_label(Some("gtk-rs"));
        p.set_website(Some("http://gtk-rs.org"));
        p.set_authors(&["Gtk-rs developers"]);
        p.set_title("About!");
        p.set_transient_for(Some(&window));
        p.show_all();
    }));

    // We need to add all the actions to the application so they can be taken into account.
    application.add_action(&about);
    application.add_action(&quit);
    application.add_action(&sub_another);
    application.add_action(&sub_sub_another);
    application.add_action(&sub_sub_another2);
    application.add_action(&switch_action);
}

fn add_accelerators(application: &gtk::Application) {
    application.set_accels_for_action("app.about", &["F1"]);
    // `Primary` is a platform-agnostic accelerator modifier.
    // On Windows and Linux, `Primary` maps to the `Ctrl` key,
    // and on macOS it maps to the `command` key.
    application.set_accels_for_action("app.quit", &["<Primary>Q"]);
}

fn build_ui(application: &gtk::Application) {
    let window = gtk::ApplicationWindow::new(application);

    window.set_title("KindNES");
    // window.set_border_width(10);
    window.set_position(gtk::WindowPosition::Center);
    window.set_default_size(256 * 3, 240 * 3);

    let v_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let label = gtk::Label::new(Some("Nothing happened yet"));
    let switch = gtk::Switch::new();
    let area = DrawingAreaBuilder::new()
        .width_request(256 * 3)
        .height_request(240 * 3)
        .can_focus(true)
        .is_focus(true)
        .has_focus(true)
        // .expand(false)
        // .opacity(0.05)
        // .hexpand(false)
        // .hexpand_set(false)
        // .vexpand(false)
        // .vexpand_set(false)
        .build();
    // gtk_widget_set_size_request((area as gtk_sys::GtkWidget).into_ptr(), 256 * 3, 240 * 3);

    v_box.pack_start(&label, false, false, 0);
    v_box.pack_start(&switch, true, true, 0);
    v_box.pack_start(&area, false, false, 0);
    // window.add(&area);
    window.add(&v_box);

    build_system_menu(application);

    add_actions(application, &switch, &label, &window);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    window.show_all();

    let wind = area.get_parent_window().unwrap();
    let mut sdl_window = unsafe {
        // let window_raw = SDL_CreateWindowFrom(&mut window.get_id() as *mut u32 as *mut c_void);
        // let window_raw = SDL_CreateWindowFrom(gdk_win32_window_get_handle(gtk_widget_get_window(window));
        // let window_raw = SDL_CreateWindowFrom(gdk_win32_window_get_handle(wind));
        let window_raw =
            SDL_CreateWindowFrom(gdk_win32_window_get_handle(area.get_window().unwrap()));
        SDL_Window::from_ll(video_subsystem, window_raw)
    };
    // sdl_window.set_size(256 * 3, 240 * 3);
    // sdl_window.set_maximum_size(256 * 3, 240 * 3);

    let mut nes = NES::new();
    nes.load_rom(File::open("D:\\Henry\\ROMs\\NES\\Contra (U).nes").unwrap())
        .unwrap();
    let mut sdl_frontend = SDLFrontend::new(sdl_context, sdl_window, Rc::new(RefCell::new(nes)));
    sdl_frontend.render_loop_with_callback(|| {
        // while gtk::events_pending() {
        gtk::main_iteration_do(false);
        // }
    });
    println!("Done?");
    //SDL_DestroyWindow(sdl_window);
    unsafe {
        SDL_Quit();
    }
    while gtk::events_pending() {
        gtk::main_iteration();
    }
}

fn main() {
    let application = gtk::Application::new(
        Some("com.github.henryksloan.kind-nes.gtk-ui"),
        Default::default(),
    )
    .expect("Initialization failed...");

    application.connect_startup(|app| {
        add_accelerators(app);
    });
    application.connect_activate(|app| {
        build_ui(app);
    });

    application.run(&args().collect::<Vec<_>>());
}
