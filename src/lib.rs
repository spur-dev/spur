use clap::Arg;
use options::{FrameRate, Quality, SType};
use std::time::{SystemTime, UNIX_EPOCH};
pub mod api;
pub mod constants;
pub mod options;
pub mod overlay;
pub mod parser;
pub mod paths;
pub mod recorder;
pub mod session;
pub mod streamer;
#[derive(Debug)]
pub enum CustomError {
    InvalidAnswer,
    CouldNotFindHome,
}

#[derive(Debug)]
pub enum ThreadMessages {
    StopPipeline,
    StartPipeline,
    PipelineStoped,
    PipelineStarted,
}

/*Config*/
#[derive(Debug, Clone)]
pub struct Config {
    pub s_type: SType,
    pub filename: Option<String>,
    // pub path: Option<String>,
    pub quality: Quality,
    pub framerate: FrameRate,
    pub overlay: bool,
    pub vid: Option<String>,
    pub uid: String,
}

// TODO: Move elsewhere
fn get_filename_for_recording(arg_filename: Option<&str>) -> String {
    let name = match arg_filename {
        Some(fname) => String::from(fname),
        None => {
            // Getting millisecond timestamp
            // https://stackoverflow.com/a/44378174/11565176
            let start = SystemTime::now();
            let since_the_epoch = start
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards");
            format!(
                "{:?}",
                since_the_epoch.as_secs() * 1000
                    + since_the_epoch.subsec_nanos() as u64 / 1_000_000
            )
        }
    };
    return format!("{}.mkv", name);
}

impl Config {
    pub fn new(uid: String, st: SType, raw_filename: Option<&str>) -> Self {
        // let path = st.get_target_path(&filename);
        let mut filename: Option<String> = None;
        if st == SType::Record {
            filename = Some(get_filename_for_recording(raw_filename))
        }

        Config {
            uid,
            filename,
            // path,
            framerate: FrameRate::default(),
            quality: Quality::default(),
            overlay: overlay::default(),
            s_type: st,
            vid: None,
        }
    }

    // CURRENTLY, NOT USED
    pub fn update_session_type(&mut self, st: SType) {
        self.s_type = st;
        // self.path = st.get_target_path(&self.filename);
    }

    pub fn get_target_path(&self) -> String {
        match self.s_type {
            SType::Record => {
                if let Some(filename) = &self.filename {
                    return paths::get_video_path(&filename)
                        .as_path()
                        .display()
                        .to_string();
                }
                panic!("Could not find target path for recording")
            }
            SType::Stream => {
                if let Some(id) = &self.vid {
                    return paths::get_stream_path(&id);
                }

                panic!("Could not find vid to start streamong")
            }
        }
    }

    pub fn create_list_arg<'a>() -> Arg<'a> {
        Arg::new("list")
            .long("list")
            .short('l')
            .takes_value(false)
            .help("Lists info of past recordings")
    }
    pub fn create_config_arg<'a>() -> Arg<'a> {
        Arg::new("config")
            .long("config")
            .short('c')
            .takes_value(false)
            .help("Show current config settings")
    }
}

impl Default for Config {
    fn default() -> Self {
        let default_st = SType::default();
        let uid = parser::get_uid();
        Config::new(uid, default_st, None)
    }
}

/*Media*/
pub trait Media {
    fn new(config: Config) -> Self
    where
        Self: Sized;
    fn start_pipeline(&mut self);
    fn stop_stream(&self);
    fn cancel_stream(&self);
    fn create_pipeline(&mut self);
}
