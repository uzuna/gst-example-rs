use derive_more::{Display, Error};
use gst::{
    prelude::{GObjectExtManualGst, GstBinExtManual},
    traits::{ElementExt, GstObjectExt},
    Element, Pipeline,
};
use std::process::Command;
use structopt::{clap::arg_enum, StructOpt};

// #[cfg(gst_app)]
mod app;

#[derive(Debug, StructOpt)]
enum Gst {
    /// launch gstremer via gst-launch-1.0
    Launch {
        #[structopt(long)]
        print: bool,
        #[structopt(flatten)]
        testsrc: VideoTestSrcOpt,
    },
    /// run gstremer with gstreamer-rs
    Run {
        #[structopt(flatten)]
        testsrc: VideoTestSrcOpt,
    },
    /// use appsrc and textoverlay
    AppSrcText {
        #[structopt(flatten)]
        testsrc: VideoTestSrcOpt,
    },
}

#[derive(Debug, StructOpt)]
struct VideoTestSrcOpt {
    #[structopt(default_value, long, possible_values = &VideoTestSrcPattern::variants())]
    pattern: VideoTestSrcPattern,
}

impl VideoTestSrcOpt {
    fn write_property_string(&self, v: &mut Vec<String>) {
        v.push(format!("pattern={}", self.pattern as u32));
    }
    fn set_properties(&self, e: &Element) {
        e.set_property_from_str("pattern", format!("{}", self.pattern as u32).as_str());
    }
}

arg_enum! {
    #[repr(u32)]
    #[derive(Debug, Clone, Copy)]
    enum VideoTestSrcPattern {
        Smpte = 0,
        Snow = 1,
        Black = 2,
        White = 3,
        Checker1 = 7,
        Checker2 = 8,
        Smpte75 = 13,
        Ball = 18,
    }
}

impl Default for VideoTestSrcPattern {
    fn default() -> Self {
        Self::Smpte
    }
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
fn build_gst_run(testsrc: &VideoTestSrcOpt) -> Result<Pipeline, anyhow::Error> {
    gst::init()?;

    // setup element
    let pipeline = gst::Pipeline::new(None);
    let src = gst::ElementFactory::make("videotestsrc").build()?;
    let sink = gst::ElementFactory::make("autovideosink").build()?;
    testsrc.set_properties(&src);

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
        Gst::Launch { print, testsrc } => {
            let mut args: Vec<String> = vec!["videotestsrc".to_string()];
            testsrc.write_property_string(&mut args);
            args.push("!".to_string());
            args.push("autovideosink".to_string());
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
        Gst::Run { testsrc } => {
            let pipeline = build_gst_run(&testsrc).expect("failed to build gst run pipeline");
            if let Err(e) = run_pipeline(pipeline) {
                log::error!("gstream error: {:?}", e);
            }
        }
        Gst::AppSrcText { testsrc } => {
            let pipeline =
                app::build_appsrc_text_overlay(&testsrc).expect("failed to build gst run pipeline");
            if let Err(e) = run_pipeline(pipeline) {
                log::error!("gstream error: {:?}", e);
            }
        }
    }
}
