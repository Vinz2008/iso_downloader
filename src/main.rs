// use std::io;
use std::ops::Deref;
use std::env;
use std::path::PathBuf;

mod debug;

use debug::print_debug;

mod downloader;

use downloader::download_isos;



#[tokio::main]
async fn main() {
    let mut is_debug = false;
    let mut config_file = None;
    let mut download_dir = env::current_dir().unwrap(); // the downloaddir is by default pwd
    let mut args = env::args();
    args.next(); // pass exe name
    while let Some(arg) = args.next() {
        if arg == "-d" {
            is_debug = true;
        } else {
            print_debug!(is_debug, "arg : {}", arg);
            if arg == "-f" {
                config_file = Some(args.next().expect("Config file path not found after '-f'"));
            } else if arg == "-D" {
                let download_dir_temp = args.next().expect("Download directory path not found after '-D'");
                download_dir  = PathBuf::from(download_dir_temp);
                //download_dir = Path::new(download_dir_temp.as_str());
            } else {
                panic!("Unexpected arg : {}", arg)
            }
        }
    }
    download_isos(config_file.expect("config file path not found"), download_dir.deref(), is_debug).await;
}
