use derive_more::{Display, Error};
use gst::{
    prelude::GstBinExtManual,
    traits::{ElementExt, GstObjectExt},
    Pipeline,
};
use std::process::Command;
use structopt::StructOpt;

// #[cfg(gst_app)]
mod app;

#[derive(Debug, StructOpt)]
enum Gst {
    /// launch gstremer via gst-launch-1.0
    Launch {
        #[structopt(long)]
        print: bool,
    },
    /// run gstremer with gstreamer-rs
    Run,
    /// use appsrc and textoverlay
    AppSrcText,
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
fn build_gst_run() -> Result<Pipeline, anyhow::Error> {
    gst::init()?;

    // setup element
    let pipeline = gst::Pipeline::new(None);
    let src = gst::ElementFactory::make("videotestsrc").build()?;
    let sink = gst::ElementFactory::make("autovideosink").build()?;

    // add to pipeline and link elements
    pipeline.add_many(&[&src, &sink])?;
    gst::Element::link_many(&[&src, &sink])?;
    Ok(pipeline)
}

fn run_pipeline(pipeline: gst::Pipeline) -> Result<(), anyhow::Error> {
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
            let pipeline = build_gst_run().expect("failed to build gst run pipeline");
            if let Err(e) = run_pipeline(pipeline) {
                log::error!("gstream error: {:?}", e);
            }
        }
        Gst::AppSrcText => {
            let pipeline =
                app::build_appsrc_text_overlay().expect("failed to build gst run pipeline");
            if let Err(e) = run_pipeline(pipeline) {
                log::error!("gstream error: {:?}", e);
            }
        }
    }
}
