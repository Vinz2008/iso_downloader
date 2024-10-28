use std::env;
use std::path::PathBuf;

pub struct Args {
    pub is_debug: bool,
    pub only_download_windows: bool,
    pub no_windows: bool,
    pub concurrent_request: u8,
    pub config_file: String,
    pub download_dir: PathBuf,
}

use crate::debug::print_debug;

pub fn parse_args() -> Args {
    let mut is_debug = false;
    let mut only_download_windows = false;
    let mut no_windows = false;
    let mut config_file = None;
    let mut download_dir = env::current_dir().unwrap(); // the downloaddir is by default pwd
    let mut args = env::args();
    let mut concurrent_request: u8 = 1;
    args.next(); // pass exe name
    while let Some(arg) = args.next() {
        if arg == "-d" {
            is_debug = true;
        } else {
            print_debug!(is_debug, "arg : {}", arg);
            if arg == "--only-windows" {
                only_download_windows = true;
            } else if arg == "--no-windows" {
                no_windows = true
            } else if arg == "-t" {
                concurrent_request = args
                    .next()
                    .expect("Thread Number not found after '-t'")
                    .parse()
                    .expect("Thread Number could not be parsed after '-t'");
            } else if arg == "-f" {
                config_file = Some(args.next().expect("Config file path not found after '-f'"));
            } else if arg == "-D" {
                let download_dir_temp = args
                    .next()
                    .expect("Download directory path not found after '-D'");
                download_dir = PathBuf::from(download_dir_temp);
                //download_dir = Path::new(download_dir_temp.as_str());
            } else {
                panic!("Unexpected arg : {}", arg)
            }
        }
    }
    return Args {
        is_debug: is_debug,
        only_download_windows: only_download_windows,
        no_windows: no_windows,
        concurrent_request: concurrent_request,
        config_file: config_file.expect("config file path not found"),
        download_dir: download_dir,
    };
}
