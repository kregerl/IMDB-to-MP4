use std::{
    fs::{File, OpenOptions},
    io::{self, Write},
    path::Path,
    thread,
};

use clap::Parser;
use vidsrc::{download_series, download_video_manifest, request_video_page, Manifest, Video};
mod vidsrc;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    movie_name: String,
    m3u8_path: String,
}

// No threading: cargo run  61.09s user 6.66s system 32% cpu 3:29.55 total
// 4 Threads: cargo run  79.47s user 13.14s system 187% cpu 49.327 total
// 8 Threads: cargo run  139.71s user 57.83s system 346% cpu 56.972 total
fn main() {
    // kung fu panda 3
    // let imdb = "tt2267968";
    // fosters
    let imdb = "tt0419326";
    // Deadpool
    // let imdb = "tt1431045";
    // Star Wars
    // let imdb = "tt0120915";
    let video = request_video_page(imdb).unwrap();
    match video {
        Video::Movie(source) => {
            let manifest = download_video_manifest(&source.title, &source.data_iframe).unwrap();
            download_video(vec![manifest]).unwrap();
        }
        Video::Series(episodes) => {
            let manifests = download_series(&episodes, |episode| {
                // episode.episode == "1" && episode.season == "1"
                true
            })
            .unwrap();
            download_video(manifests).unwrap();
        }
    }
}

fn download_video(manifests: Vec<Manifest>) -> io::Result<()> {
    for manifest in manifests {
        let movie_file_name = format!("{}.mp4", manifest.title);
        let movie_path = Path::new(&movie_file_name);
        if movie_path.exists() {
            println!("File with that name already exists '{}'", movie_file_name);
            return Ok(());
        }

        let mut urls: Vec<String> = Vec::new();
        for line in manifest.index.lines() {
            if line.starts_with("https") {
                urls.push(line.to_owned());
            }
        }
        thread::scope(|scope| {
            const MAX_CHUNKS: usize = 4;
            let mut thread_handles = Vec::with_capacity(MAX_CHUNKS);
            let chunks = urls.chunks((urls.len() as f32 / MAX_CHUNKS as f32).ceil() as usize);
            for (id, chunk) in chunks.enumerate() {
                thread_handles
                    .push(scope.spawn(move || download_movie_chunk(id + 1, chunk).unwrap()))
            }
            File::create(movie_path).unwrap();
            let mut file = OpenOptions::new().append(true).open(movie_path).unwrap();
            for (index, thread) in thread_handles.into_iter().enumerate() {
                println!("Joined chunk {}", index + 1);
                let bytes = thread.join().expect("Error joining thread");
                file.write_all(&bytes).unwrap();
            }
        });
    }
    Ok(())
}

fn download_movie_chunk(id: usize, url_chunk: &[String]) -> Result<Vec<u8>, String> {
    let mut all_bytes = Vec::new();
    for (index, url) in url_chunk.iter().enumerate() {
        println!(
            "Downloading {}/{} in chunk {}",
            index + 1,
            url_chunk.len(),
            id
        );
        let response = reqwest::blocking::get(url).expect("Error downloading video part.");
        if !response.status().is_success() {
            return Err(format!("Bad status code: {:#?}", response.status()));
        }

        let bytes = response.bytes().unwrap();
        all_bytes.append(&mut bytes.to_vec());
    }

    Ok(all_bytes)
}
