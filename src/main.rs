use derive_more::{Display, Error};
use gst::{
    prelude::GstBinExtManual,
    traits::{ElementExt, GstObjectExt},
};
use std::process::Command;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
enum Gst {
    /// launch gstremer via gst-launch-1.0
    Launch {
        #[structopt(long)]
        print: bool,
    },
    /// run gstremer with gstreamer-rs
    Run,
}

#[derive(Debug, Display, Error)]
#[display(fmt = "Received error from {}: {} (debug: {:?})", src, error, debug)]
struct ErrorMessage {
    src: String,
    error: String,
    debug: Option<String>,
    source: gst::glib::Error,
}

/// run gstreamer
fn gst_main_loop() -> Result<(), anyhow::Error> {
    gst::init()?;

    // setup element
    let pipeline = gst::Pipeline::new(None);
    let src = gst::ElementFactory::make("videotestsrc").build()?;
    let sink = gst::ElementFactory::make("autovideosink").build()?;

    // add to pipeline and link elements
    pipeline.add_many(&[&src, &sink])?;
    gst::Element::link_many(&[&src, &sink])?;

    // start playing
    pipeline.set_state(gst::State::Playing)?;
    let bus = pipeline
        .bus()
        .expect("Pipeline without bus. Shouldn't happen!");

    for msg in bus.iter_timed(gst::ClockTime::NONE) {
        use gst::MessageView;

        match msg.view() {
            MessageView::Eos(..) => break,
            MessageView::Error(err) => {
                pipeline.set_state(gst::State::Null)?;
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
            _ => (),
        }
    }

    pipeline.set_state(gst::State::Null)?;
    Ok(())
}

fn main() {
    let opt = Gst::from_args();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    match opt {
        Gst::Launch { print } => {
            let args = ["videotestsrc", "!", "autovideosink"];
            if print {
                println!("gst-launch-1.0 {}", args.join(" "));
            } else {
                let mut process = match Command::new("gst-launch-1.0")
                    .args(args)
                    .stdout(std::process::Stdio::inherit())
                    .spawn()
                {
                    Err(why) => panic!("couldn't spawn wc: {}", why),
                    Ok(process) => process,
                };

                if let Err(e) = process.wait() {
                    log::error!("command error: {}", e);
                }
            }
        }
        Gst::Run => {
            if let Err(e) = gst_main_loop() {
                log::error!("gstream error: {:?}", e);
            }
        }
    }
}
