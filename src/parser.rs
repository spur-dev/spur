use crate::{
    constants,
    options::{FrameRate, MetaOption, Quality, SType},
    overlay, parser, paths,
    session::Session,
    Config,
};
use clap::{Arg, ArgMatches, Command};
use std::{
    fs,
    io::Write,
    process,
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

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

pub fn get_filename_from_arg(st: SType, arg_filename: Option<&str>) -> String {
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
    match st {
        SType::Record => format!("{}.mkv", name),
        SType::Stream => format!("/{}/{}", env!("STREAM_API_PATH"), name),
    }
}

fn setup() {
    // Creating a Videos directory
    let videos_path = paths::get_video_directory_path();

    fs::DirBuilder::new()
        .recursive(true)
        .create(&videos_path)
        .unwrap_or_else(|err| {
            println!("Couldn't create directory - {}", videos_path.display());
            println!("Error - {:?}", err);
            process::exit(1);
        });

    // CURRENTLY, NOT USED
    // Saving conf to text file
    let home_path = paths::get_home_path().expect("Couldn't find your Home directory");
    let mut conf_file = fs::File::create(format!(
        "{}/{}",
        home_path.display(),
        constants::CONFIG_FILE_NAME
    ))
    .expect("Could not create conf file");
    conf_file
        .write_all(format!("{}", videos_path.display()).as_bytes()) // hacky
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
    match matches.subcommand() {
        Some(("setup", _)) => {
            setup();
            process::exit(0);
        }
        Some((cmd_str, sub_match)) => {
            // Creating config for new session
            let st = SType::from_str(cmd_str).unwrap();
            let arg_filename = sub_match.value_of("name");
            let mut conf = Config::new(st, get_filename_from_arg(st, arg_filename));

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
