mod debug;
mod downloader;
mod args;
mod progress_bar;

use downloader::download_isos;

use args::parse_args;

#[tokio::main]
async fn main() {
    let args = parse_args();
    download_isos(args).await;
}
