pub mod window;

use gtk::prelude::*;
use gtk::{gio, glib};
use window::Window;

fn main() -> glib::ExitCode {
    gio::resources_register_include!("nvogue-gui.gresource").unwrap();

    let app = adw::Application::builder()
        .application_id("org.mads256h.NVogue")
        .build();

    app.connect_activate(build_ui);

    app.run()
}

fn build_ui(app: &adw::Application) {
    let window = Window::new(app);
    window.present();
}
