use derive_more::{Display, Error};
use gst::{
    prelude::{ElementExtManual, GObjectExtManualGst, GstBinExtManual},
    traits::{ElementExt, GstBinExt, GstObjectExt},
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
        #[structopt(flatten)]
        videocaps: VideoCapsOpt,
    },
    /// run gstremer with gstreamer-rs
    Run {
        #[structopt(flatten)]
        testsrc: VideoTestSrcOpt,
        #[structopt(flatten)]
        videocaps: VideoCapsOpt,
    },
    /// use appsrc and textoverlay
    AppSrcText {
        #[structopt(flatten)]
        testsrc: VideoTestSrcOpt,
        #[structopt(flatten)]
        videocaps: VideoCapsOpt,
    },
    /// use gstbin
    GstBin {
        #[structopt(flatten)]
        videocaps: VideoCapsOpt,
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

#[derive(Debug, StructOpt)]
struct VideoCapsOpt {
    // 画面の横幅
    #[structopt(default_value = "600", long)]
    width: i32,
    // 画面の高さ
    #[structopt(default_value = "400", long)]
    height: i32,
    // フレームレート(整数のみ)
    #[structopt(default_value = "30", long)]
    fps: i32,
}

impl Default for VideoCapsOpt {
    fn default() -> Self {
        Self {
            width: 600,
            height: 400,
            fps: 30,
        }
    }
}

impl VideoCapsOpt {
    fn write_property_string(&self, v: &mut Vec<String>) {
        v.push(format!(
            "video/x-raw,width={},height={},framerate={}/1",
            self.width, self.height, self.fps
        ));
    }
    fn get_caps(&self) -> gst::Caps {
        // gst_video::Caps::VideoCapsBuilderを使うともっと直感的に書ける
        // i32なのはC実装が受け付ける型がi32であるため他の型ではエラーになる
        gst::Caps::builder("video/x-raw")
            .field("width", self.width)
            .field("height", self.height)
            .field("framerate", gst::Fraction::new(self.fps, 1))
            .build()
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
fn build_gst_run(
    testsrc: &VideoTestSrcOpt,
    videocaps: &VideoCapsOpt,
) -> Result<Pipeline, anyhow::Error> {
    gst::init()?;

    // setup element
    let pipeline = gst::Pipeline::new(None);
    let src = gst::ElementFactory::make("videotestsrc").build()?;
    let sink = gst::ElementFactory::make("autovideosink").build()?;
    testsrc.set_properties(&src);

    // add to pipeline and link elements
    pipeline.add_many(&[&src, &sink])?;
    // attach caps
    src.link_filtered(&sink, &videocaps.get_caps())?;
    Ok(pipeline)
}

/// use bin in application
pub(crate) fn build_bin() -> Result<gst::Pipeline, anyhow::Error> {
    gst::init()?;

    // build bin
    let bin = gst::Bin::new(Some("example_bin"));
    let videosrc = gst::ElementFactory::make("videotestsrc").build()?;
    let sink = gst::ElementFactory::make("autovideosink").build()?;
    bin.add_many(&[&videosrc, &sink])?;
    videosrc.link(&sink)?;

    let pipeline = gst::Pipeline::default();
    pipeline.add(&bin)?;
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
        Gst::Launch {
            print,
            testsrc,
            videocaps,
        } => {
            let mut args: Vec<String> = vec!["videotestsrc".to_string()];
            testsrc.write_property_string(&mut args);
            args.push("!".to_string());
            videocaps.write_property_string(&mut args);
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
        Gst::Run { testsrc, videocaps } => {
            let pipeline =
                build_gst_run(&testsrc, &videocaps).expect("failed to build gst run pipeline");
            if let Err(e) = run_pipeline(pipeline) {
                log::error!("gstream error: {:?}", e);
            }
        }
        Gst::AppSrcText { testsrc, videocaps } => {
            let pipeline = app::build_appsrc_text_overlay(&testsrc, &videocaps)
                .expect("failed to build gst run pipeline");
            if let Err(e) = run_pipeline(pipeline) {
                log::error!("gstream error: {:?}", e);
            }
        }
        Gst::GstBin { .. } => {
            let pipeline = build_bin().expect("failed to build gst run pipeline");
            if let Err(e) = run_pipeline(pipeline) {
                log::error!("gstream error: {:?}", e);
            }
        }
    }
}
