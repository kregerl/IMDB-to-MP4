use std::{io, process::Command, string::FromUtf8Error};

use indicatif::{ProgressBar, ProgressStyle};
use regex::Regex;
use reqwest::header::{HeaderName, HeaderValue};
use scraper::{error::SelectorErrorKind, Selector};

use crate::scraper::{get_document, parse_attribute, parse_inner_html};


#[derive(Debug)]
pub struct Manifest {
    pub title: String,
    pub index: String,
    pub season: Option<String>,
    pub episode: Option<String>,
}

pub fn download_series<'a>(episodes: &'a [Episode], episode_filter: fn(&Episode) -> bool) -> VidsrcResult<'a, Vec<Manifest>> {
    let episodes = episodes.iter().filter(|episode| episode_filter(episode)).collect::<Vec<_>>();
    let length = episodes.len();
    let mut manifests = Vec::with_capacity(length);

    let pb = ProgressBar::new(length as u64);
    let sty = ProgressStyle::with_template(
        "{msg} {bar:40.cyan/blue} {pos:>7}/{len:7} [{elapsed_precise}]",
    )
    .unwrap()
    .progress_chars("##-");
    pb.set_style(sty);
    pb.println("Downloading Manifests...");
    for episode in episodes {
        pb.set_message(episode.title.clone());
        let video_source = request_video_source(&episode.data_iframe)?;
        let mut manifest = download_video_manifest(&video_source.title, &video_source.data_iframe)?;
        manifest.episode = Some(episode.episode.clone());
        manifest.season = Some(episode.season.clone());
        manifests.push(manifest);
        pb.inc(1);
    }
    pb.finish();
    Ok(manifests)
}

pub fn download_video_manifest<'a>(title: &str, iframe_url: &str) -> VidsrcResult<'a, Manifest> {
    let hash_parts = request_hash_page(&iframe_url).unwrap();

    let encoded_file_id_url_part = spawn_js_worker(
        "src/js/encoded_file_id_parser.js",
        &[hash_parts.0, hash_parts.1],
    )
    .unwrap();
    let encoded_file_id_url = format!("https:{}", encoded_file_id_url_part);
    let encoded_file_id = request_encoded_file_id(&encoded_file_id_url).unwrap();

    let decoded_index_url =
        spawn_js_worker("src/js/decode_file_id.js", &[encoded_file_id]).unwrap();
    // println!("decoded_index_url: {}", decoded_index_url);
    
    let data = reqwest::blocking::get(decoded_index_url.trim())?;

    Ok(Manifest {
        title: title.to_owned(),
        index: data.text()?,
        season: None,
        episode: None,
    })
}

#[derive(Debug)]
enum JSError {
    Worker(io::Error),
    StringFmt(FromUtf8Error),
    Js(String),
}

impl From<io::Error> for JSError {
    fn from(value: io::Error) -> Self {
        Self::Worker(value)
    }
}

impl From<FromUtf8Error> for JSError {
    fn from(value: FromUtf8Error) -> Self {
        Self::StringFmt(value)
    }
}
type JSResult<T> = Result<T, JSError>;

fn spawn_js_worker(program_name: &str, args: &[String]) -> JSResult<String> {
    let child = Command::new("nodejs")
        .arg(program_name)
        .args(args)
        .output()?;

    let stdout = String::from_utf8(child.stdout)?;
    let stderr = String::from_utf8(child.stderr)?;

    if !stderr.is_empty() {
        return Err(JSError::Js(stderr));
    }

    Ok(stdout)
}

#[derive(Debug)]
pub enum VidsrcError<'a> {
    Request(reqwest::Error),
    Selector(SelectorErrorKind<'a>),
    EmptySelector,
    EmptyAttr,
    InvalidFileId,
}

impl<'a> From<reqwest::Error> for VidsrcError<'a> {
    fn from(value: reqwest::Error) -> Self {
        Self::Request(value)
    }
}

impl<'a> From<SelectorErrorKind<'a>> for VidsrcError<'a> {
    fn from(value: SelectorErrorKind<'a>) -> Self {
        Self::Selector(value)
    }
}

pub type VidsrcResult<'a, T> = Result<T, VidsrcError<'a>>;

#[derive(Debug)]
pub struct Episode {
    title: String,
    data_iframe: String,
    pub season: String,
    pub episode: String
}

#[derive(Debug)]
pub struct VideoSource {
    pub title: String,
    pub data_iframe: String,
}

#[derive(Debug)]
pub enum Video {
    Movie(VideoSource),
    Series(Vec<Episode>),
}

const VIDSRC_BASE_URL: &str = "https://vidsrc.xyz";
pub fn request_video_page(imdb: &str) -> VidsrcResult<Video> {
    let url = format!("{}/embed/{}", VIDSRC_BASE_URL, imdb);

    let document = get_document(&url)?;

    let selector = Selector::parse("div.ep[data-iframe]")?;
    let elements = document.select(&selector).collect::<Vec<_>>();
    Ok(if elements.len() == 0 {
        let iframe_url_part = parse_attribute(&document, "#player_iframe", "src")?;
        let iframe_url = format!("https:{}", iframe_url_part);

        let title = parse_inner_html(&document, "title")?;
        Video::Movie(VideoSource {
            title,
            data_iframe: iframe_url,
        })
    } else {
        let mut episodes = Vec::with_capacity(elements.len());
        for tag in document.select(&selector) {
            let iframe_attr = tag.attr("data-iframe").ok_or(VidsrcError::EmptyAttr)?;
            let season_attr = tag.attr("data-s").ok_or(VidsrcError::EmptyAttr)?;
            let episode_attr = tag.attr("data-e").ok_or(VidsrcError::EmptyAttr)?;
            let title = tag.inner_html();
            episodes.push(Episode {
                title,
                data_iframe: iframe_attr.into(),
                season: season_attr.into(),
                episode: episode_attr.into(),
            })
        }
        Video::Series(episodes)
    })
}

fn request_video_source(endpoint: &str) -> VidsrcResult<VideoSource> {
    let url = format!("{}{}", VIDSRC_BASE_URL, endpoint);
    let document = get_document(&url)?;

    let iframe_url_part = parse_attribute(&document, "#player_iframe", "src")?;
    let iframe_url = format!("https:{}", iframe_url_part);

    let title = parse_inner_html(&document, "title")?;
    Ok(VideoSource {
        title,
        data_iframe: iframe_url,
    })
}



fn request_hash_page(url: &str) -> VidsrcResult<(String, String)> {
    let response = reqwest::blocking::Client::new()
        .get(url)
        .header(
            HeaderName::from_static("referer"),
            HeaderValue::from_static("https://vidsrc.xyz/"),
        )
        .send()?;
    let html = response.text()?;
    let document = scraper::Html::parse_document(&html);
    let data_i = parse_attribute(&document, "body[data-i]", "data-i")?;
    let data_h = parse_attribute(&document, "div[data-h]", "data-h")?;

    Ok((data_i.into(), data_h.into()))
}

fn request_encoded_file_id(url: &str) -> VidsrcResult<String> {
    let response = reqwest::blocking::Client::new()
        .get(url)
        .header(
            HeaderName::from_static("referer"),
            HeaderValue::from_static("https://vidsrc.xyz/"),
        )
        .send()?;
    let html = response.text()?;
    let document = scraper::Html::parse_document(&html);
    let inner_html = parse_inner_html(&document, "script:not([src])")?;

    let prefix = "file:\"";
    let suffix = "\",";
    let re: Regex = Regex::new(&format!("{}?.*{}", prefix, suffix)).unwrap();
    let encoded_file_id_str: &str = re.find(&inner_html).unwrap().as_str();
    let encoded_file_id: &str = (|| {
        encoded_file_id_str
            .strip_prefix(prefix)?
            .strip_suffix(suffix)
    })()
    .ok_or(VidsrcError::InvalidFileId)?;

    Ok(encoded_file_id.into())
}

