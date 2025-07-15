use std::cmp::min;

use indicatif::{MultiProgress,  ProgressStyle};

// TODO : add a struct with all the state
// TODO : add a label with the total speed ?


pub struct ProgressBar {
    progress_bar : indicatif::ProgressBar,
    url : String,
    out_path_str : String,
}

impl ProgressBar {
    pub fn new(multi_progress : Option<&MultiProgress>, url : &str, out_path_str: &str, total_size : u64, download_name : Option<&str>) -> ProgressBar {
        let pb = match multi_progress {
            Some(mpb) => mpb.add(indicatif::ProgressBar::new(total_size)),
            None => indicatif::ProgressBar::new(total_size)
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

        ProgressBar {
            progress_bar : pb,
            url : url.to_owned(),
            out_path_str: out_path_str.to_owned()
        }
    }

    pub fn update(&self, chunk_len : usize){
        let old_pos = self.progress_bar.position();
        let pb_length = self.progress_bar.length().unwrap();
        let new_pos = min(old_pos + (chunk_len as u64), pb_length);
        self.progress_bar.set_position(new_pos);
    }

    pub fn finish(self){
        self.progress_bar.finish_with_message(format!("Downloaded {} to {}", self.url, self.out_path_str));
    }
}

// TODO : make these member functions



