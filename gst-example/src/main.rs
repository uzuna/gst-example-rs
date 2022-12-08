use derive_more::{Display, Error};
use gst::{
    prelude::{ElementExtManual, GObjectExtManualGst, GstBinExtManual},
    traits::{ElementExt, GstBinExt, GstObjectExt},
    Element, Pipeline,
};
use signal_hook::flag;

use std::{
    process::Command,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use structopt::{clap::arg_enum, StructOpt};

// #[cfg(gst_app)]
mod app;
mod probe;

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
    /// probe sample
    Probe {
        #[structopt(flatten)]
        testsrc: VideoTestSrcOpt,
        #[structopt(flatten)]
        videocaps: VideoCapsOpt,
    },
    /// probe tee
    ProbeTee {
        #[structopt(flatten)]
        testsrc: VideoTestSrcOpt,
        #[structopt(flatten)]
        videocaps: VideoCapsOpt,
        #[structopt(long)]
        usequeue: bool,
    },
    /// probe tsdemux
    ProbeTsdemux {
        #[structopt(flatten)]
        testsrc: VideoTestSrcOpt,
        #[structopt(flatten)]
        videocaps: VideoCapsOpt,
        #[structopt(long)]
        savefile: Option<String>,
    },
}

#[derive(Debug, StructOpt)]
struct VideoTestSrcOpt {
    #[structopt(default_value, long, possible_values = &VideoTestSrcPattern::variants())]
    pattern: VideoTestSrcPattern,
    #[structopt(default_value = "-1", long)]
    num_buffers: i64,
}

impl VideoTestSrcOpt {
    fn write_property_string(&self, v: &mut Vec<String>) {
        v.push(format!("pattern={}", self.pattern as u32));
    }
    fn set_properties(&self, e: &Element) {
        e.set_property_from_str("pattern", format!("{}", self.pattern as u32).as_str());
        e.set_property_from_str(
            "num-buffers",
            format!("{}", self.num_buffers as u32).as_str(),
        );
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
pub(crate) fn build_bin(videocaps: &VideoCapsOpt) -> Result<gst::Pipeline, anyhow::Error> {
    gst::init()?;

    // build bin
    let bin = gst::Bin::new(Some("example_bin"));
    let videosrc = gst::ElementFactory::make("videotestsrc").build()?;
    let sink = gst::ElementFactory::make("autovideosink").build()?;
    bin.add_many(&[&videosrc, &sink])?;
    // attach caps
    videosrc.link_filtered(&sink, &videocaps.get_caps())?;

    let pipeline = gst::Pipeline::default();
    pipeline.add(&bin)?;
    Ok(pipeline)
}

fn run_pipeline(pipeline: gst::Pipeline) -> Result<(), anyhow::Error> {
    let term = Arc::new(AtomicBool::new(false));
    flag::register(signal_hook::consts::SIGTERM, Arc::clone(&term))?;
    flag::register(signal_hook::consts::SIGINT, Arc::clone(&term))?;

    // start playing
    pipeline.set_state(gst::State::Playing)?;
    let bus = pipeline
        .bus()
        .expect("Pipeline without bus. Shouldn't happen!");

    // Ctrl+Cでパイプラインを停止する
    'outer: while !term.load(Ordering::Relaxed) {
        // シグナルを定期的に確認する
        for msg in bus.iter_timed(gst::ClockTime::MSECOND * 100) {
            use gst::MessageView;

            match msg.view() {
                MessageView::Eos(..) => break 'outer,
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

    pipeline.set_state(gst::State::Null)?;
    let (res, ..) = pipeline.state(Some(gst::ClockTime::SECOND));
    log::info!("shutdown pipeline {:?}", res.unwrap());

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
        Gst::GstBin { videocaps } => {
            let pipeline = build_bin(&videocaps).expect("failed to build gst run pipeline");
            if let Err(e) = run_pipeline(pipeline) {
                log::error!("gstream error: {:?}", e);
            }
        }
        Gst::Probe { testsrc, videocaps } => {
            let pipeline = probe::build_pipeline(&testsrc, &videocaps)
                .expect("failed to build gst run pipeline");
            if let Err(e) = run_pipeline(pipeline) {
                log::error!("gstream error: {:?}", e);
            }
        }
        Gst::ProbeTee {
            testsrc,
            videocaps,
            usequeue,
        } => {
            let pipeline = probe::build_tee_probe(&testsrc, &videocaps, usequeue)
                .expect("failed to build gst run pipeline");
            if let Err(e) = run_pipeline(pipeline) {
                log::error!("gstream error: {:?}", e);
            }
        }
        Gst::ProbeTsdemux {
            testsrc,
            videocaps,
            savefile,
        } => {
            let pipeline = probe::build_demux_probe(&testsrc, &videocaps, savefile.as_ref())
                .expect("failed to build gst run pipeline");
            if let Err(e) = run_pipeline(pipeline) {
                log::error!("gstream error: {:?}", e);
            }
        }
    }
}
