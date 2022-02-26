use clap::Arg;
use options::{FrameRate, Quality, SType};
pub mod constants;
pub mod options;
pub mod overlay;
pub mod parser;
pub mod recorder;
pub mod session;
pub mod streamer;

#[derive(Debug)]
pub enum CustomError {
    InvalidAnswer,
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
    pub name: String,
    pub cmd: &'static str,
    pub path: &'static str,
    pub quality: Quality,
    pub framerate: FrameRate,
    pub overlay: bool,
}

impl Config {
    pub fn new(st: SType) -> Self {
        let (name, path) = match st {
            // Make tis a function
            SType::Record => (SType::to_string(&SType::Record), "rtmp://someendpoint:1935"),
            SType::Stream => (SType::to_string(&SType::Stream), "my-demo.mkv"),
        };
        Config {
            cmd: constants::CMD,
            name,
            path,
            framerate: FrameRate::default(),
            quality: Quality::default(),
            overlay: overlay::default(),
            s_type: st,
        }
    }
    pub fn update_session_type(&mut self, st: SType) {
        self.s_type = st;
        let (name, path) = match st {
            SType::Record => (SType::to_string(&SType::Record), "rtmp://someendpoint:1935"),
            SType::Stream => (SType::to_string(&SType::Stream), "my-demo.mkv"),
        };
        self.name = name;
        self.path = path;
    }
    pub fn create_path_arg<'a>(&self) -> Arg<'a> {
        Arg::new("path")
            .long("path")
            .takes_value(self.s_type == SType::Record)
            .default_value(self.path)
            .help("Set path of recording")
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

impl Default for Config {
    fn default() -> Self {
        Config::new(SType::default())
    }
}
