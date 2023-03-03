use env_logger::Env;
use gst::prelude::{Cast, ObjectExt};
use gst::traits::{ElementExt, GstObjectExt};
use gst::MessageView;
use gtk::{prelude::*, DrawingArea, Orientation};
use gtk::{Application, ApplicationWindow, Button};

fn run() -> anyhow::Result<()> {
    gst::init()?;

    let uri =
        "https://www.freedesktop.org/software/gstreamer-sdk/data/media/sintel_trailer-480p.webm";
    let playbin = gst::ElementFactory::make("playbin").build()?;
    playbin.set_property("uri", &uri);
    let pipeline = playbin.downcast::<gst::Pipeline>().unwrap();
    let bus = pipeline
        .bus()
        .expect("Pipeline without bus. Shouldn't happen!");
    let pipeline_weak = pipeline.downgrade();
    bus.connect_message(None, move |_, msg| {
        let pipeline = match pipeline_weak.upgrade() {
            Some(pipeline) => pipeline,
            None => return,
        };

        match msg.view() {
            gst::MessageView::Eos(..) => {
                println!("End of stream reached");
                pipeline
                    .set_state(gst::State::Ready)
                    .expect("Unable to set pipeline to the ready state");
            }
            MessageView::Error(err) => {
                pipeline.set_state(gst::State::Null).unwrap();
                log::debug!(
                    "error [{}] {:?}",
                    err.src()
                        .map(|s| String::from(s.path_string()))
                        .unwrap_or_else(|| String::from("None")),
                    err.error().to_string(),
                );
            }
            MessageView::StateChanged(x) => {
                log::debug!(
                    "state-changed [{}] {:?} -> {:?}",
                    x.src().unwrap().name(),
                    x.old(),
                    x.current()
                );
            }
            _ => {}
        }
    });

    let application = Application::builder()
        .application_id("com.example.FirstGtkApp")
        .build();

    let pipeline_weak = pipeline.downgrade();
    application.connect_activate(move |app| {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("First GTK Program")
            .default_width(600)
            .default_height(400)
            .build();

        let play_button =
            Button::from_icon_name(Some("media-playback-start"), gtk::IconSize::SmallToolbar);
        let p2 = pipeline_weak.clone();
        play_button.connect_clicked(move |_| {
            log::info!("Play!");
            let pipeline = match p2.upgrade() {
                Some(pipeline) => pipeline,
                None => return,
            };
            pipeline.set_state(gst::State::Playing).unwrap();
        });
        let p2 = pipeline_weak.clone();
        let stop_button =
            Button::from_icon_name(Some("media-playback_stop"), gtk::IconSize::SmallToolbar);
        stop_button.connect_clicked(move |_| {
            log::info!("Stop!");
            let pipeline = match p2.upgrade() {
                Some(pipeline) => pipeline,
                None => return,
            };
            pipeline.set_state(gst::State::Null).unwrap();
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
    pipeline.set_state(gst::State::Null)?;
    Ok(())
}

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    run().unwrap()
}
