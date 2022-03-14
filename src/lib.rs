use clap::Arg;
use options::{FrameRate, Quality, SType};
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
    pub filename: String,
    pub path: String,
    pub quality: Quality,
    pub framerate: FrameRate,
    pub overlay: bool,
}

impl Config {
    pub fn new(st: SType, filename: String) -> Self {
        let path = st.get_target_path(&filename);

        Config {
            filename: String::from(filename),
            path,
            framerate: FrameRate::default(),
            quality: Quality::default(),
            overlay: overlay::default(),
            s_type: st,
        }
    }

    // CURRENTLY, NOT USED
    pub fn update_session_type(&mut self, st: SType) {
        self.s_type = st;
        self.path = st.get_target_path(&self.filename);
    }
    pub fn create_path_arg(&self) -> Arg {
        Arg::new("path")
            .long("path")
            .takes_value(self.s_type == SType::Record)
            .default_value(self.path.as_str())
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

impl Default for Config {
    fn default() -> Self {
        let default_st = SType::default();
        Config::new(default_st, parser::get_filename_from_arg(default_st, None))
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
