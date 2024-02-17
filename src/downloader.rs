use std::{fs::{self, File, OpenOptions}, io::{self, Write}, path::{Path, PathBuf}, thread};

use crate::{scraper::{get_document, parse_inner_html}, vidsrc::{download_series, download_video_manifest, request_video_page, Manifest, Video, VidsrcResult}};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use reqwest::Url;

pub fn extract_imdb_id(url_str: &str) -> Option<String> {
    let url_str = url_str.strip_suffix("/").unwrap_or(url_str);
    let url = Url::parse(url_str).ok()?;
    Some(url.path_segments()?.last()?.to_owned())
}

const IMDB_BASE_URL: &str = "https://www.imdb.com/title";
pub fn download_video_from_imdb_id(imdb_id: &str) {
    let video = request_video_page(imdb_id).unwrap();
    match video {
        Video::Movie(source) => {
            let manifest = download_video_manifest(&source.title, &source.data_iframe).unwrap();
            download_video(vec![manifest], None).unwrap();
        }
        Video::Series(episodes) => {
            let imdb_url = format!("{}/{}", IMDB_BASE_URL, imdb_id);
            let series_title = get_series_title(&imdb_url).unwrap();

            let manifests = download_series(&episodes, |episode| {
                //TODO: Let this support arbitrary conditions defined in the yaml
                // true
                episode.season == "3" && (episode.episode == "7" || episode.episode == "8") 
            })
            .unwrap();
            let series_dir_path = Path::new(&series_title).to_path_buf();
            download_video(manifests, Some(series_dir_path)).unwrap();
        }
    }
}

fn get_series_title(imdb_url: &str) -> VidsrcResult<String> {
    let document = get_document(&imdb_url)?;
    let title = parse_inner_html(&document, "span.hero__primary-text")?;

    Ok(title)
}

fn download_video(manifests: Vec<Manifest>, base_dir: Option<PathBuf>) -> io::Result<()> {
    for manifest in manifests {
        let mb = MultiProgress::new();
        mb.println(format!("\nDownloading '{}'", manifest.title))?;
        let movie_file_name = format!("{}.mp4", manifest.title);
        let movie_path = if let Some(season) = manifest.season {
            manifest.title.rsplit_once(" ");
            let dir_name = &format!("Season {:0>2}", season);
            let dir_path = match base_dir.as_ref() {
                Some(base) => base.join(Path::new(dir_name)),
                None => Path::new(dir_name).to_path_buf(),
            };
            fs::create_dir_all(&dir_path)?;

            dir_path.join(&movie_file_name)
        } else {
            Path::new(&movie_file_name).to_path_buf()
        };

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

        let sty = ProgressStyle::with_template(
            "{msg} {bar:40.cyan/blue} {pos:>7}/{len:7} [{elapsed_precise}]",
        )
        .unwrap()
        .progress_chars("##-");
        thread::scope(|scope| {
            const MAX_CHUNKS: usize = 4;
            let mut thread_handles = Vec::with_capacity(MAX_CHUNKS);
            let chunk_size = (urls.len() as f32 / MAX_CHUNKS as f32).ceil() as usize;

            let chunks = urls.chunks(chunk_size);
            for (id, chunk) in chunks.enumerate() {
                let pb = mb.add(ProgressBar::new(chunk.len() as u64));
                pb.set_style(sty.clone());
                thread_handles
                    .push(scope.spawn(move || download_movie_chunk(id, pb, chunk).unwrap()))
            }
            mb.println("").unwrap();
            mb.clear().unwrap();
            File::create(&movie_path).unwrap();
            let mut file = OpenOptions::new().append(true).open(movie_path).unwrap();
            for thread in thread_handles.into_iter() {
                let bytes = thread.join().expect("Error joining thread");
                file.write_all(&bytes).unwrap();
            }
        });
    }
    Ok(())
}

fn download_movie_chunk(
    id: usize,
    pb: ProgressBar,
    url_chunk: &[String],
) -> Result<Vec<u8>, String> {
    let mut all_bytes = Vec::new();
    pb.set_message(format!("Thread {}", id + 1));
    for (_index, url) in url_chunk.iter().enumerate() {
        let response = reqwest::blocking::get(url).expect("Error downloading video part.");
        if !response.status().is_success() {
            return Err(format!("Bad status code: {:#?}", response.status()));
        }

        let bytes = response.bytes().unwrap();
        all_bytes.append(&mut bytes.to_vec());
        pb.inc(1);
    }
    pb.finish();
    Ok(all_bytes)
}
