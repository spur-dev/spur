use crate::{constants, CustomError};
use std::path::PathBuf;

pub fn get_home_path() -> Result<PathBuf, CustomError> {
    match home::home_dir() {
        Some(path) => Ok(path),
        // None => process::exit(1),
        None => Err(CustomError::CouldNotFindHome),
    }
}

pub fn get_video_directory_path() -> PathBuf {
    let mut path = get_home_path().expect("Couldn't find your Home directory");
    path.push(constants::VIDEOS_FOLDER_FROM_HOME);
    path
}

pub fn get_video_path(filename: &String) -> PathBuf {
    // get from conf file
    let mut path = get_video_directory_path();
    path.push(filename);
    path
}

pub fn get_stream_path(vid: &String) -> String {
    format!("{}/{}", env!("STREAM_API"), vid)
}
