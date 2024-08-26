use std::fs::read_to_string;
use std::fs::File;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::cmp::min;
use std::io::Write;
use std::process::Command;
use std::env;

use reqwest::Client;
use indicatif::{ProgressBar, ProgressStyle};
use toml::{Table, map};
use url::Url;
use futures_util::StreamExt;

use crate::debug::print_debug;

async fn download_file_in_path(client: &Client, download_name : Option<&str>, url : &str, out_path : &Path) -> Result<(), String> {
    let resp = client.get(url).send().await.or(Err(format!("Failed to GET from '{}'", &url)))?;
    let total_size = resp.content_length().ok_or(format!("Failed to get content length from '{}'", &url))?;
    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::default_bar()
        .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})").unwrap()
        .progress_chars("#>-"));
    let pb_message = if download_name.is_none(){
        format!("Downloading {} {}...", "mido", url)
    } else {
        format!("Downloading {} {}...", download_name.unwrap(), url)
    };
    pb.set_message(pb_message);
    let out_path_str = out_path.to_str().or(Some("")).unwrap();
    let mut file = File::create(out_path).or(Err(format!("Failed to create file '{}'", out_path_str)))?;
    let mut downloaded: u64 = 0;
    let mut stream = resp.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item.or(Err(format!("Error while downloading file")))?;
        file.write_all(&chunk)
            .or(Err(format!("Error while writing to file")))?;
        let new = min(downloaded + (chunk.len() as u64), total_size);
        downloaded = new;
        pb.set_position(new);
    }

    pb.finish_with_message(format!("Downloaded {} to {}", url, out_path_str));
    Ok(())

    
    /*let resp = reqwest::blocking::get(url).expect("request failed");
    let body = resp.text().expect("body invalid");
    let mut out = File::create(out_path).expect("failed to create file");
    io::copy(&mut body.as_bytes(), &mut out).expect("Failed to copy content");*/
}

fn get_last_item_iter<I : IntoIterator, F : FnMut(&I::Item) -> bool,>(iter : I, mut f : F) -> Option<I::Item> {
    let mut last = None;
    
    for elem in iter {
        if f(&elem){
            return Some(elem);
        }
        last = Some(elem);
    }
    
    last
}

async fn download_file(client: &Client, download_name : Option<&str>, url : &str, dir : &Path, is_debug : bool) -> Result<(), String> {
    let parsed_url = Url::parse(url).expect("Invalid url");
    let url_segments = parsed_url.path_segments().unwrap();
    let url_filename = get_last_item_iter(url_segments, |&item| {
        item.ends_with(".iso")
    }).unwrap();
    let mut out_path = PathBuf::new();
    out_path.push(dir);
    out_path.push(Path::new(url_filename));
    print_debug!(is_debug, "out_path : {}", out_path.to_str().unwrap());
    download_file_in_path(client, download_name, url, out_path.deref()).await
}


// TODO : add validation for keys
// TODO : add parallel download

pub async fn download_mido_script(client: &Client, is_debug : bool) -> PathBuf{
    let mut out_path: PathBuf = env::temp_dir();
    out_path.push("mido.sh");
    let url = "https://raw.githubusercontent.com/ElliotKillick/Mido/main/Mido.sh";
    download_file_in_path(client, None, url, out_path.deref()).await.expect("Couldn't find the mido script");
    if cfg!(unix){
        print_debug!(is_debug, "out_path : {}", out_path.to_str().unwrap());
        Command::new("chmod").arg("+x").arg(out_path.to_str().unwrap()).output().expect("failed to execute chmod +x on the mido script");
    }
    out_path
}

async fn download_windows_isos(client: &Client, windows_isos : &map::Map<String, toml::Value>, is_debug : bool){
    let script_path = download_mido_script(client, is_debug).await;
    let windows_versions_vals = windows_isos["windows_versions"].as_array().expect("Windows versions should be an array");
    let windows_versions = windows_versions_vals.into_iter().map(|version| version.as_str().expect("Windows versions should be strings")).collect::<Vec<&str>>();
    Command::new(script_path).args(windows_versions).output().expect("failed to execute the mido script");
    /*for version in windows_versions {
        let version_str = version.as_str().expect("Windows versions should be strings");
    }*/
}

pub async fn download_isos(config_file : String, download_dir : &Path, is_debug : bool){
    print_debug!(is_debug, "config_file : {}", config_file);
    let file_content = read_to_string(config_file).expect("Config file not found");
    let table = file_content.parse::<Table>().unwrap();
    print_debug!(is_debug, "{}", table);
    let client = reqwest::Client::new();
    let downloads = table["downloads"].as_table().expect("The downloads table is missing");
    for download in downloads {
        //println!("downloading {}...", download.0);
        let download_url : &str = download.1.as_str().expect("Urls of downloads should be strings");
        //let download_url = download.1.as_str().expect("Urls of downloads should be strings");
        download_file(&client, Some(download.0.as_str()), download_url, download_dir, is_debug).await.expect("Couldn't download file");
    }
    let windows_isos = table["windows_downloads"].as_table();
    // TODO
    /*if windows_isos.is_some(){
        let windows_isos = windows_isos.unwrap();
        download_windows_isos(&client, windows_isos, is_debug).await;
    }*/
}
