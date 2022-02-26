use crate::CustomError;
use clap::Arg;
use std::str::FromStr;
pub trait MetaOption {
    const COMMAND_NAME: &'static str;
    fn values() -> [&'static str; 2];
    fn create_arg<'a>() -> Arg<'a>;
}

/** Quality */
#[derive(Debug, Clone, Copy)]
pub enum Quality {
    Q720 = 720,
    Q1080 = 1080,
}
impl Default for Quality {
    fn default() -> Self {
        Quality::Q720
    }
}

impl FromStr for Quality {
    type Err = CustomError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "720" => Ok(Quality::Q720),
            "1080" => Ok(Quality::Q1080),
            _ => Err(CustomError::InvalidAnswer),
        }
    }
}

impl ToString for Quality {
    fn to_string(&self) -> String {
        match self {
            &Self::Q720 => String::from("720"),
            &Self::Q1080 => String::from("1080"),
        }
    }
}
impl MetaOption for Quality {
    const COMMAND_NAME: &'static str = "quality";
    fn values() -> [&'static str; 2] {
        ["720", "1080"]
    }

    fn create_arg<'a>() -> Arg<'a> {
        Arg::new(Self::COMMAND_NAME)
            .long(Self::COMMAND_NAME)
            .takes_value(true)
            .default_value("720")
            .required(false)
            .help("The quality of the recording")
    }
}

/** Framerate */
#[derive(Debug, Clone, Copy)]
pub enum FrameRate {
    F24 = 24 as isize,
    F30 = 30 as isize,
}

impl Default for FrameRate {
    fn default() -> Self {
        FrameRate::F24
    }
}

impl FromStr for FrameRate {
    type Err = CustomError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "24" => Ok(FrameRate::F24),
            "30" => Ok(FrameRate::F30),
            _ => Err(CustomError::InvalidAnswer),
        }
    }
}

impl ToString for FrameRate {
    fn to_string(&self) -> String {
        match self {
            &Self::F24 => String::from("24"),
            &Self::F30 => String::from("30"),
        }
    }
}

impl MetaOption for FrameRate {
    const COMMAND_NAME: &'static str = "framerate";
    fn values() -> [&'static str; 2] {
        ["24", "30"]
    }

    fn create_arg<'a>() -> Arg<'a> {
        Arg::new(Self::COMMAND_NAME)
            .long(Self::COMMAND_NAME)
            .takes_value(true)
            .default_value("24")
            .required(false)
            .help("Framerate of recording")
    }
}

/** Type */
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum SType {
    Record = 0,
    Stream = 1,
}

impl Default for SType {
    fn default() -> Self {
        SType::Stream
    }
}

impl FromStr for SType {
    type Err = CustomError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "record" => Ok(SType::Record),
            "stream" => Ok(SType::Stream),
            _ => Err(CustomError::InvalidAnswer),
        }
    }
}

impl ToString for SType {
    fn to_string(&self) -> String {
        match self {
            &Self::Record => String::from("record"),
            &Self::Stream => String::from("stream"),
        }
    }
}
impl MetaOption for SType {
    const COMMAND_NAME: &'static str = "session";
    fn values() -> [&'static str; 2] {
        const SUB_COMMANDS: [&'static str; 2] = ["record", "stream"];
        SUB_COMMANDS
    }

    fn create_arg<'a>() -> Arg<'a> {
        Arg::new(Self::COMMAND_NAME)
            .long(Self::COMMAND_NAME)
            .takes_value(true)
            .default_value("record")
            .required(false)
            .help("Wether to stream to server or store to storage")
    }
}
