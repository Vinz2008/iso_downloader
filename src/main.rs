mod debug;

mod downloader;

use downloader::download_isos;

mod args;

use args::parse_args;

#[tokio::main]
async fn main() {
    let args = parse_args();
    download_isos(args).await;
}
