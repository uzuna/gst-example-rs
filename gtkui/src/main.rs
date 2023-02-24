use env_logger::Env;
use gtk::{prelude::*, DrawingArea, Orientation};
use gtk::{Application, ApplicationWindow, Button};

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let application = Application::builder()
        .application_id("com.example.FirstGtkApp")
        .build();

    application.connect_activate(|app| {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("First GTK Program")
            .default_width(600)
            .default_height(400)
            .build();

        let play_button =
            Button::from_icon_name(Some("media-playback-start"), gtk::IconSize::SmallToolbar);
        play_button.connect_clicked(|_| {
            log::info!("Play!");
        });
        let stop_button =
            Button::from_icon_name(Some("media-playback_stop"), gtk::IconSize::SmallToolbar);
        stop_button.connect_clicked(|_| {
            log::info!("Stop!");
        });
        window.connect_delete_event(|_, _| {
            log::info!("Close!");
            gtk::main_quit();
            Inhibit(false)
        });

        let controls = gtk::Box::new(Orientation::Horizontal, 0);
        controls.pack_start(&play_button, false, false, 0);
        controls.pack_start(&stop_button, false, false, 0);
        let video_window = DrawingArea::new();
        let videos = gtk::Box::new(Orientation::Horizontal, 0);
        videos.pack_start(&video_window, false, false, 0);
        let main_box = gtk::Box::new(Orientation::Vertical, 0);
        main_box.pack_start(&controls, false, false, 0);
        main_box.pack_start(&videos, false, false, 0);
        window.add(&main_box);

        window.show_all();
    });

    let code = application.run();
    println!("exit {:?}", code);
}
