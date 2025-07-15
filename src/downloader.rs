use std::fs::{read_to_string, File};
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::io::Write;
use std::process::Command;
use std::env;
use std::process::Stdio;
use std::sync::Arc;

use indicatif::MultiProgress;
use reqwest::Client;
use toml::{Table, map};
use url::Url;
use futures_util::{stream, StreamExt};

use crate::args::Args;
use crate::debug::print_debug;
use crate::progress_bar::{ProgressBar};



// TODO : better error handling ?
async fn download_file_in_path(client: &Client, download_name : Option<&str>, url : &str, out_path : &Path, multi_progress : Option<&MultiProgress>) -> Result<(), String> {
    let resp = client.get(url).send().await.map_err(|e| format!("Failed to GET from '{}, {}'", &url, e))?;
    let total_size = resp.content_length().ok_or_else(|| format!("Failed to get content length from '{}'", &url))?;
    
    let out_path_str = out_path.to_str().expect("The out path is not UTF-8");
    let mut file = File::create(out_path).unwrap_or_else(|e| panic!("Failed to create file '{}', {}", out_path_str, e));
    let mut stream = resp.bytes_stream();

    let pb = ProgressBar::new(multi_progress, url, out_path_str, total_size, download_name);

    while let Some(item) = stream.next().await {
        let chunk = item.or(Err("Error while downloading file"))?;
        file.write_all(&chunk)
            .or(Err("Error while writing to file".to_owned()))?;
        pb.update(chunk.len());
    }

    pb.finish();
    Ok(())

}


async fn download_file(client: &Client, download_name : Option<&str>, url : &str, dir : &Path, is_debug : bool, multi_progress : Option<&MultiProgress>) -> Result<(), String> {
    // TODO : create url parser and replace this
    let parsed_url = Url::parse(url).expect("Invalid url");
    let url_segments = parsed_url.path_segments().unwrap();

    // use next_back instead of last to not needlessly iterate
    let mut url_filename = url_segments.into_iter().next_back().unwrap();
    
    if url_filename.contains("?"){
        url_filename = url_filename.split_once("?").unwrap().0;
    }

    if !url_filename.ends_with(".iso"){
        return Err(format!("The file {} that is tried to be download is not an iso", url_filename));
    }

    let mut out_path = PathBuf::new();
    out_path.push(dir);
    out_path.push(Path::new(url_filename));
    print_debug!(is_debug, "out_path : {}", out_path.to_str().unwrap());
    if !out_path.exists(){
        download_file_in_path(client, download_name, url, out_path.deref(), multi_progress).await
    } else {
        Ok(())
    }
    
}


// TODO : add validation for keys
// TODO : add retries for downloads
// TODO : automatically find the version to download for isos ()

pub async fn download_mido_script(client: &Client, is_debug : bool) -> PathBuf {
    let mut out_path: PathBuf = env::temp_dir();
    out_path.push("mido.sh");
    let url = "https://raw.githubusercontent.com/ElliotKillick/Mido/main/Mido.sh";
    download_file_in_path(client, None, url, out_path.deref(), None).await.expect("Couldn't find the mido script");
    if cfg!(unix){
        print_debug!(is_debug, "out_path : {}", out_path.to_str().unwrap());
        Command::new("chmod").arg("+x").arg(out_path.to_str().unwrap()).output().expect("failed to execute chmod +x on the mido script");
    }
    out_path
}

async fn download_windows_isos(client: &Client, windows_isos : &map::Map<String, toml::Value>, is_debug : bool, download_dir : &Path){
    let script_path = download_mido_script(client, is_debug).await;
    let windows_versions_vals = windows_isos["windows_versions"].as_array().expect("Windows versions should be an array");
    let windows_versions = windows_versions_vals.iter().map(|version| version.as_str().expect("Windows versions should be strings")).collect::<Vec<&str>>();
    Command::new(script_path)
        .args(windows_versions)
        .current_dir(download_dir.canonicalize().unwrap())
        .stderr(Stdio::inherit())
        .spawn().expect("failed to execute the mido script")
        .wait().unwrap();
}


pub async fn download_isos(args : Args){
    print_debug!(args.is_debug, "config_file : {}", args.config_file);
    let file_content = read_to_string(args.config_file).expect("Config file not found");
    let table = file_content.parse::<Table>().unwrap();
    print_debug!(args.is_debug, "{}", table);
    
    let client = reqwest::Client::new();
    
    if !args.only_download_windows {

        let downloads = table.get("downloads").expect("The downloads table is missing").as_table().expect("The downloads table is not table");
        if args.concurrent_request == 1 {
            for download in downloads {
                let download_url = download.1.as_str().expect("Urls of downloads should be strings");
                let download_name = download.0;
                download_file(&client, Some(download_name), download_url, &args.download_dir , args.is_debug, None).await.expect("Couldn't download file");
            }
        } else {
        
        
        let multi_progress = Arc::new(MultiProgress::new());
        let downloads_iter = stream::iter(downloads.iter()).map(|download| {
            let cloned_client = client.clone();
            let multi_progress = Arc::clone(&multi_progress);
            let download_name = Arc::new(download.0.to_owned());
            let download_url = Arc::new(download.1.as_str().expect("Urls of downloads should be strings").to_owned());
            let cloned_download_dir = args.download_dir.to_owned();
            tokio::spawn(async move {
                let client = &cloned_client;
                download_file(client, Some((*download_name).as_str()), (*download_url).as_str(), &cloned_download_dir, args.is_debug, Some(&multi_progress)).await.expect("Couldn't download file");
            })
        }).buffer_unordered(args.concurrent_request as usize);

        downloads_iter.for_each(|join_result| async {
            match join_result {
                Ok(()) => {},
                Err(e) => panic!("Download task failed {}", e)
            }
        }).await;

        }

    }

    if !args.no_windows {
        let windows_isos = table["windows_downloads"].as_table();
        // TODO
        if let Some(windows_isos) = windows_isos{
            download_windows_isos(&client, windows_isos, args.is_debug, &args.download_dir).await;
        }
    }
}
