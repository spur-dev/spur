use crate::{
    constants,
    options::{FrameRate, MetaOption, Quality, SType},
    overlay, parser, paths,
    session::Session,
    Config,
};

use clap::{Arg, ArgMatches, Command};
use std::{
    fs::{self, File},
    io::{prelude::*, BufReader, Write},
    process,
    str::FromStr,
};
use uuid::Uuid;

/* Utils */
fn create_sub_command(st: SType) -> Command<'static> {
    Command::new(st.get_name()).args([
        overlay::create_arg(),
        Quality::create_arg(),
        FrameRate::create_arg(),
        Arg::new("name")
            .long("filename")
            .short('n')
            .takes_value(true)
            .help("Name of recorded video"),
    ])
}

fn generate_uuid() -> String {
    return Uuid::new_v4().to_string();
}

// TODO: File path for read could be improved
pub fn get_uid() -> String {
    let home_path = paths::get_home_path().expect("Couldn't find your Home directory");
    let conf_file = File::open(format!(
        "{}/{}",
        home_path.display(),
        constants::CONFIG_FILE_NAME
    ))
    .unwrap();

    let mut uid = String::new();
    BufReader::new(conf_file)
        .read_line(&mut uid)
        .expect("Could not find UID, Please Run Setup");
    return uid;
}

fn setup() {
    // Creating a Videos directory
    let videos_path = paths::get_video_directory_path();
    let uid = generate_uuid();
    fs::DirBuilder::new()
        .recursive(true)
        .create(&videos_path)
        .unwrap_or_else(|err| {
            println!("Couldn't create directory - {}", videos_path.display());
            println!("Error - {:?}", err);
            process::exit(1);
        });

    // Saving conf to text file
    let home_path = paths::get_home_path().expect("Couldn't find your Home directory");
    let mut conf_file = fs::File::create(format!(
        "{}/{}",
        home_path.display(),
        constants::CONFIG_FILE_NAME
    ))
    .expect("Could not create conf file");
    conf_file
        // .write_all(format!("{} \n{} \n", uid, videos_path.display()).as_bytes()) // hacky
        .write_all(uid.as_bytes())
        .expect("Could not write to conf file");
    println!("-------------- Setup is complete ------------------------");
}

/* Parser */
pub fn parse_args() -> ArgMatches {
    return Command::new("spur")
        .version(constants::VERSION)
        .about(constants::DESCRIPTION)
        .author(constants::AUTHOR)
        .args([Config::create_list_arg(), Config::create_config_arg()])
        .subcommands([
            create_sub_command(SType::Record),
            create_sub_command(SType::Stream),
            Command::new("setup").about("setting up spur on your machine"),
        ])
        .get_matches();
}

pub fn create_session_from_args() -> Session {
    let matches = parser::parse_args();
    let uid = parser::get_uid();
    match matches.subcommand() {
        Some(("setup", _)) => {
            setup();
            process::exit(0);
        }
        Some((cmd_str, sub_match)) => {
            // Creating config for new session
            let st = SType::from_str(cmd_str).unwrap();
            let arg_filename = sub_match.value_of("name");
            let mut conf = Config::new(uid, st, arg_filename);

            // Updating config with parsed parameters
            let arg_quality = sub_match
                .value_of(Quality::COMMAND_NAME)
                .unwrap_or_default();
            conf.quality = Quality::from_str(arg_quality).expect("Unable to parse arg - quality");
            let arg_framerate = sub_match
                .value_of(FrameRate::COMMAND_NAME)
                .unwrap_or_default();
            conf.framerate =
                FrameRate::from_str(arg_framerate).expect("Unable to parse arg - framerate");

            let arg_overlay = sub_match
                .value_of(overlay::COMMAND_NAME)
                .unwrap_or_default();
            conf.overlay = !(arg_overlay == "false" || arg_overlay == "0");

            Session::new(conf)
        }
        None => Session::default(),
    }
}
