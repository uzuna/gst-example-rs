use std::{process, ffi::c_void};

use derive_more::{Display, Error};
use gst::{prelude::*, glib};
use gst_video::prelude::VideoOverlayExtManual;
use gtk::{prelude::*, DrawingArea, Orientation};
use gdk::prelude::*;
// use gtk::{Window, WindowType, traits::{WidgetExt, ButtonExt, BoxExt}, Inhibit, Orientation, DrawingArea};


#[derive(Debug, Display, Error)]
#[display(fmt = "Received error from {}: {} (debug: {:?})", src, error, debug)]
struct ErrorMessage {
    src: String,
    error: String,
    debug: Option<String>,
    source: gst::glib::Error,
}


fn create_ui(playbin: &gst::Element) {
    // Instanciate window, button, sliders and register their event handlers
    let main_window = Window::new(WindowType::Toplevel);
    main_window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });
    let pipeline = playbin.clone();
    let play_button = gtk::Button::from_icon_name(
        Some("media-playback-start"),
        gtk::IconSize::SmallToolbar,
    );
    play_button.connect_clicked(move |_| {
        // Add event handler to the event  when the button was clicked
        let pipeline = &pipeline;
        pipeline
            .set_state(gst::State::Playing)
            .expect("Unable to set the pipeline to `Playing` state");
    });

    let pause_button = gtk::Button::from_icon_name(
        Some("media-playback-pause"),
        gtk::IconSize::SmallToolbar,
    );
    let pipeline = playbin.clone();
    pause_button.connect_clicked(move |_| {
        // Add event handler to the event  when the button was clicked
        let pipeline = &pipeline;

        pipeline
            .set_state(gst::State::Paused)
            .expect("Unable to set the pipeline to the `Paused` state");
    });

    let stop_button = gtk::Button::from_icon_name(
        Some("media-playback_stop"),
        gtk::IconSize::SmallToolbar,
    );
    let pipeline = playbin.clone();
    stop_button.connect_clicked(move |_| {
        // Add event handler to the event when the button was clicked
        let pipeline = &pipeline;
        pipeline
            .seek_simple(
                gst::SeekFlags::FLUSH | gst::SeekFlags::KEY_UNIT,
                0 * gst::ClockTime::MSECOND,
            )
            .expect("Failed to seek to start");
        pipeline
            .set_state(gst::State::Paused)
            .expect("Unable to set the pipeline to the `Ready` state");
    });
    

    let controls = gtk::Box::new(Orientation::Horizontal,0);
    controls.pack_start(&play_button, false, false, 0);
    controls.pack_start(&pause_button, false, false, 0);
    controls.pack_start(&stop_button, false, false, 0);


    // Create video area
    let video_window = DrawingArea::new();
    let video_overlay = playbin
        .clone()
        .dynamic_cast::<gst_video::VideoOverlay>()
        .unwrap();
    video_window.connect_realize(move |video_window| {
        let video_overlay = &video_overlay;
        let gdk_window = video_window.get_window().unwrap();

        if !gdk_window.ensure_native() {
            println!("Can't create native window for widget");
            process::exit(-1);
        }

        let display_type_name = gdk_window.get_display().get_type().name();
        if display_type_name == "GdkX11Display" {
            extern "C" {
                pub fn gdk_x11_window_get_xid(
                    window: *mut glib::object::Object,
                ) -> *mut c_void;
            }

            #[allow(clippy::cast_ptr_alignment)]
            unsafe {
                // Call native API to obtain the window pointer
                let xid = gdk_x11_window_get_xid(gdk_window.as_ptr() as *mut _);
                // Set destination with the handler
                video_overlay.set_window_handle(xid as usize);
            }
        } else {
            println!("Add support for display type {}", display_type_name);
            process::exit(-1);
        }
    });

}

fn run() -> Result<(), anyhow::Error> {
    gst::init()?;
    gtk::init()?;

    let uri = "https://www.freedesktop.org/software/gstreamer-sdk/data/media/sintel_trailer-480p.webm";
    let playbin = gst::ElementFactory::make("playbin").build()?;
    playbin.set_property("uri", &uri);

    playbin
    .connect("video-tags-changed", false, |args| {
        print!("video-tags-changed");
        None
    });
    create_ui(&playbin);

    playbin.set_state(gst::State::Playing)?;
    let bus = playbin
        .bus()
        .expect("Pipeline without bus. Shouldn't happen!");

    // Ctrl+Cでパイプラインを停止する
    'outer: loop {
        // シグナルを定期的に確認する
        for msg in bus.iter_timed(gst::ClockTime::MSECOND * 100) {
            use gst::MessageView;

            match msg.view() {
                MessageView::Eos(..) => break 'outer,
                MessageView::Error(err) => {
                    playbin.set_state(gst::State::Null)?;
                    let error = err.error().to_string();
                    if error.contains("Output window was closed") {
                        break;
                    }
                    return Err(ErrorMessage {
                        src: msg
                            .src()
                            .map(|s| String::from(s.path_string()))
                            .unwrap_or_else(|| String::from("None")),
                        error: err.error().to_string(),
                        debug: err.debug(),
                        source: err.error(),
                    }
                    .into());
                }
                MessageView::StateChanged(x) => {
                    log::debug!(
                        "state-changed [{}] {:?} -> {:?}",
                        x.src().unwrap().name(),
                        x.old(),
                        x.current()
                    );
                }
                x => {
                    log::debug!("message {:?}", x);
                }
            }
        }
    }

    gtk::main();

    Ok(())
}

fn main() {
    run().unwrap();
}
