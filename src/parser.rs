use crate::{
    constants,
    options::{FrameRate, MetaOption, Quality, SType},
    overlay, parser,
    session::Session,
    Config,
};
use clap::{ArgMatches, Command};
use std::str::FromStr;

fn create_sub_command<'a>(st: SType) -> Command<'a> {
    let conf = Config::new(st);
    let path = conf.create_path_arg();

    Command::new(conf.name).args([
        overlay::create_arg(),
        Quality::create_arg(),
        FrameRate::create_arg(),
        path,
    ])
}

pub fn parse_args() -> ArgMatches {
    return Command::new("spur")
        .version(constants::VERSION)
        .about(constants::DESCRIPTION)
        .author(constants::AUTHOR)
        .args([Config::create_list_arg(), Config::create_config_arg()])
        .subcommands([
            create_sub_command(SType::Record),
            create_sub_command(SType::Stream),
        ])
        .get_matches();
}

pub fn create_session_from_args() -> Session {
    let mut conf = Config::default();
    let matches = parser::parse_args();
    match matches.subcommand() {
        Some((cmd_str, sub_match)) => {
            conf.s_type = SType::from_str(cmd_str).unwrap();
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
