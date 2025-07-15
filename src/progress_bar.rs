use std::cmp::min;

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

pub fn start_progress_bar(multi_progress : Option<&MultiProgress>, url : &str, total_size : u64, download_name : Option<&str>) -> ProgressBar {
    let pb = match multi_progress {
        Some(mpb) => mpb.add(ProgressBar::new(total_size)),
        None => ProgressBar::new(total_size)
    };
    pb.set_style(ProgressStyle::default_bar()
        .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})").unwrap()
        .progress_chars("#>-"));

    // TODO : instead of Option str, use an enum with one Mido, and the other the name
    let pb_message = if download_name.is_none(){
        format!("Downloading mido {}...", url)
    } else {
        format!("Downloading {} {}...", download_name.unwrap(), url)
    };

    pb.set_message(pb_message);
    pb
}

// call in the loop
pub fn handle_progress_bar(pb : &ProgressBar, pb_pos : &mut u64, chunk_len : usize, total_size : u64){
    *pb_pos = min(*pb_pos + (chunk_len as u64), total_size);
    pb.set_position(*pb_pos);
}

pub fn finish_progress_bar(pb : &ProgressBar, url : &str, out_path_str : &str){
    pb.finish_with_message(format!("Downloaded {} to {}", url, out_path_str));
}