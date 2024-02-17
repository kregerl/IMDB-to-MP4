use clap::{Arg, Command};
use downloader::{download_video_from_imdb_id, extract_imdb_id};
use serde::Deserialize;
use std::{fs::File, io::BufReader, process::exit};

mod downloader;
mod scraper;
mod vidsrc;

#[derive(Deserialize, Debug)]
struct Videos {
    urls: Vec<String>,
}

// No threading: cargo run  61.09s user 6.66s system 32% cpu 3:29.55 total
// 4 Threads: cargo run  79.47s user 13.14s system 187% cpu 49.327 total
// 8 Threads: cargo run  139.71s user 57.83s system 346% cpu 56.972 total
fn main() {
    let args = Command::new("IMDB2MP4")
        .arg(
            Arg::new("from_url")
                .short('u')
                .long("from-url")
                .exclusive(true),
        )
        .arg(
            Arg::new("from_file")
                .short('f')
                .long("from-file")
                .exclusive(true),
        )
        .get_matches();

    if let Some(url_str) = args.get_one::<String>("from_url") {
        let imdb_id = extract_imdb_id(&url_str);
        if imdb_id.is_none() {
            eprintln!("Invalid IMDB URL.");
            exit(1);
        }
        let imdb = imdb_id.unwrap();
        download_video_from_imdb_id(&imdb);
    }

    if let Some(file) = args.get_one::<String>("from_file") {
        let file = File::open(file).unwrap();
        let reader = BufReader::new(file);
        let videos: Videos = serde_yaml::from_reader(reader).unwrap();
        for video_url in videos.urls {
            let imdb_id = extract_imdb_id(&video_url);
            if imdb_id.is_none() {
                eprintln!("Invalid IMDB URL {}", video_url);
                continue;
            }
            download_video_from_imdb_id(&imdb_id.unwrap());
        }
    }
}
