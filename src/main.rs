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
        Gst::Run => todo!(),
    }
}
