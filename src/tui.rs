use std::{
    fs::{self, File, OpenOptions},
    io::{self, stdout, Read, Stdout, Write},
    path::{Component, Path},
    sync::{Arc, Mutex, RwLock},
    thread,
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Flex, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Text,
    widgets::{Block, Borders, Gauge, List, ListDirection, Paragraph},
    Frame, Terminal,
};

use crate::{
    text_input::{self, TextInput},
    vidsrc::{download_video_manifest, Manifest, VideoSource},
};

pub struct App<'a> {
    title: Option<String>,
    percentages: [Arc<RwLock<u16>>; 4],
    text_input: TextInput<'a>,
}

impl<'a> App<'a> {
    pub fn new() -> Self {
        Self {
            title: None,
            percentages: Default::default(),
            text_input: TextInput::new("IMDB Url"),
        }
    }
}

pub fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    sources: Vec<VideoSource>,
) -> io::Result<()> {
    let titles = sources
        .iter()
        .map(|s| s.title.clone())
        .collect::<Vec<_>>();
    for source in sources {
        let manifest = download_video_manifest(&source.title, &source.data_iframe).unwrap();
        
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
        app.title = Some(manifest.title.clone());
        thread::scope(|scope| {
            const MAX_CHUNKS: usize = 4;
            let mut thread_handles = Vec::with_capacity(MAX_CHUNKS);
            let chunks = urls.chunks((urls.len() as f32 / MAX_CHUNKS as f32).ceil() as usize);
            for (id, chunk) in chunks.enumerate() {
                let pct = app.percentages[id].clone();
                thread_handles
                    .push(scope.spawn(move || download_movie_chunk(id + 1, chunk, pct).unwrap()))
            }
            let mut should_quit = false;
            while !should_quit {
                terminal.draw(|frame| ui(frame, &mut app, &titles)).unwrap();
                should_quit = handle_events(&mut app).unwrap();

                if thread_handles.iter().all(|handle| handle.is_finished()) {
                    should_quit = true;
                }
            }

            File::create(movie_path).unwrap();
            let mut file = OpenOptions::new().append(true).open(movie_path).unwrap();
            for (index, thread) in thread_handles.into_iter().enumerate() {
                println!("Joined chunk {}", index + 1);
                let bytes = thread.join().expect("Error joining thread");
                file.write_all(&bytes).unwrap();
            }
        });
        app.title = None;
    }

    Ok(())
}

fn handle_events(app: &mut App) -> io::Result<bool> {
    if event::poll(std::time::Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                return Ok(false);
            }

            match key.code {
                KeyCode::Enter => {
                    
                    app.text_input.clear();
                },
                KeyCode::Esc => return Ok(true),
                _ => {}
            }
            app.text_input.on_input(key.code);
        }
    }
    Ok(false)
}

fn ui(frame: &mut Frame, app: &mut App, titles: &[String]) {
    let [header, content] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .areas(frame.size());

    Renderer::render_header(frame, header, app);

    let [status, queue]: [Rect; 2] = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Fill(0), Constraint::Fill(0)])
        .areas(content);

    Renderer::render_percentages(frame, status, app);
    Renderer::render_queue(frame, queue, titles);
}

struct Renderer;

impl Renderer {
    fn render_header(frame: &mut Frame, area: Rect, app: &App) {
        frame.set_cursor(
            area.x + app.text_input.cursor_position() as u16 + 1,
            area.y + 1,
        );
        frame.render_widget(
            app.text_input
                .clone()
                .block(Block::default().borders(Borders::ALL)),
            area,
        );
    }

    fn render_percentages(frame: &mut Frame, area: Rect, app: &App) {
        let [header, body] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .areas(area);

        if let Some(title) = &app.title {
            frame.render_widget(
                Paragraph::new(title.as_str()).block(Block::default().borders(Borders::ALL)),
                header,
            );
        }

        let body_areas: [Rect; 4] = Layout::default()
            .flex(Flex::Legacy)
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(0),
                Constraint::Fill(0),
                Constraint::Fill(0),
                Constraint::Fill(0),
            ])
            .areas(body);

        Renderer::render_percentage_body(frame, body_areas, &app.percentages);
    }

    fn render_percentage_body(
        frame: &mut Frame,
        areas: [Rect; 4],
        percentages: &[Arc<RwLock<u16>>; 4],
    ) {
        for (i, percentage) in percentages.iter().enumerate() {
            frame.render_widget(
                Gauge::default()
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(format!("Thread {}", i + 1)),
                    )
                    .gauge_style(
                        Style::default()
                            .fg(Color::White)
                            .bg(Color::Black)
                            .add_modifier(Modifier::ITALIC),
                    )
                    .percent(*percentage.read().as_deref().unwrap()),
                areas[i],
            );
        }
    }

    fn render_queue(frame: &mut Frame, area: Rect, titles: &[String]) {
        let list = List::new(
            titles
                .iter()
                .map(|title| title.as_str())
                .collect::<Vec<_>>(),
        )
        .block(Block::default().borders(Borders::ALL))
        .direction(ListDirection::TopToBottom);

        frame.render_widget(list, area);
    }
}

fn download_movie_chunk(
    id: usize,
    url_chunk: &[String],
    percent: Arc<RwLock<u16>>,
) -> Result<Vec<u8>, String> {
    let mut all_bytes = Vec::new();
    for (index, url) in url_chunk.iter().enumerate() {
        // println!(
        //     "Downloading {}/{} in chunk {}",
        //     index + 1,
        //     url_chunk.len(),
        //     id
        // );
        let response = reqwest::blocking::get(url).expect("Error downloading video part.");
        if !response.status().is_success() {
            return Err(format!("Bad status code: {:#?}", response.status()));
        }

        let bytes = response.bytes().unwrap();
        all_bytes.append(&mut bytes.to_vec());

        let mut lock = percent.write().unwrap();
        *lock = (((index + 1) as f32 / url_chunk.len() as f32) * 100f32) as u16;
    }

    Ok(all_bytes)
}

// fn main() -> io::Result<()> {
//     enable_raw_mode()?;
//     stdout().execute(EnterAlternateScreen)?;
//     let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

//     let mut file = File::open("./indexes/deadpool.m3u8")?;
//     let mut manifest = String::new();
//     file.read_to_string(&mut manifest)?;
//     let movie_file_name = format!("{}.mp4", "Deadpool");
//     let movie_path = Path::new(&movie_file_name);
//     if movie_path.exists() {
//         println!("File with that name already exists '{}'", movie_file_name);
//         return Ok(());
//     }

//     let mut urls: Vec<String> = Vec::new();
//     for line in manifest.lines() {
//         if line.starts_with("https") {
//             urls.push(line.to_owned());
//         }
//     }

//     thread::scope(|scope| {
//         let percentages: [Arc<RwLock<u16>>; 4] = Default::default();

//         const MAX_CHUNKS: usize = 4;
//         let mut thread_handles = Vec::with_capacity(MAX_CHUNKS);
//         let chunks = urls.chunks((urls.len() as f32 / MAX_CHUNKS as f32).ceil() as usize);
//         for (id, chunk) in chunks.enumerate() {
//             let pct = percentages[id].clone();
//             thread_handles
//                 .push(scope.spawn(move || download_movie_chunk(id + 1, chunk, pct).unwrap()))
//         }
//         let mut should_quit = false;
//         while !should_quit {
//             let pcts = percentages.clone();
//             terminal
//                 .draw(move |frame| {
//                     ui(frame, pcts);
//                 })
//                 .unwrap();
//             should_quit = handle_events().unwrap();

//             if thread_handles.iter().all(|handle| handle.is_finished()) {
//                 should_quit = true;
//             }
//         }

//         File::create(movie_path).unwrap();
//         let mut file = OpenOptions::new().append(true).open(movie_path).unwrap();
//         for (index, thread) in thread_handles.into_iter().enumerate() {
//             println!("Joined chunk {}", index + 1);
//             let bytes = thread.join().expect("Error joining thread");
//             file.write_all(&bytes).unwrap();
//         }
//     });

//     disable_raw_mode()?;
//     stdout().execute(LeaveAlternateScreen)?;
//     Ok(())
// }
// fn handle_events() -> io::Result<bool> {
//     if event::poll(std::time::Duration::from_millis(50))? {
//         if let Event::Key(key) = event::read()? {
//             if key.kind == event::KeyEventKind::Press && key.code == KeyCode::Char('q') {
//                 return Ok(true);
//             }
//         }
//     }
//     Ok(false)
// }

// fn ui(frame: &mut Frame, percentages: [Arc<RwLock<u16>>; 4]) {
//     let layout: [Rect; 4] = Layout::default()
//         .flex(Flex::Legacy)
//         .direction(Direction::Vertical)
//         .constraints([
//             Constraint::Fill(0),
//             Constraint::Fill(0),
//             Constraint::Fill(0),
//             Constraint::Fill(0),
//         ])
//         .areas(frame.size());

//     frame.render_widget(
//         Gauge::default()
//             .block(Block::default().borders(Borders::ALL).title("Progress"))
//             .gauge_style(
//                 Style::default()
//                     .fg(Color::White)
//                     .bg(Color::Black)
//                     .add_modifier(Modifier::ITALIC),
//             )
//             .percent(*percentages[0].read().as_deref().unwrap()),
//         layout[0],
//     );

//     frame.render_widget(
//         Gauge::default()
//             .block(Block::default().borders(Borders::ALL).title("Progress"))
//             .gauge_style(
//                 Style::default()
//                     .fg(Color::White)
//                     .bg(Color::Black)
//                     .add_modifier(Modifier::ITALIC),
//             )
//             .percent(*percentages[1].read().as_deref().unwrap()),
//         layout[1],
//     );

//     frame.render_widget(
//         Gauge::default()
//             .block(Block::default().borders(Borders::ALL).title("Progress"))
//             .gauge_style(
//                 Style::default()
//                     .fg(Color::White)
//                     .bg(Color::Black)
//                     .add_modifier(Modifier::ITALIC),
//             )
//             .percent(*percentages[2].read().as_deref().unwrap()),
//         layout[2],
//     );

//     frame.render_widget(
//         Gauge::default()
//             .block(Block::default().borders(Borders::ALL).title("Progress"))
//             .gauge_style(
//                 Style::default()
//                     .fg(Color::White)
//                     .bg(Color::Black)
//                     .add_modifier(Modifier::ITALIC),
//             )
//             .percent(*percentages[3].read().as_deref().unwrap()),
//         layout[3],
//     );

//     // frame.render_widget(
//     //     Gauge::default()
//     //         .block(Block::default().borders(Borders::ALL).title("Progress"))
//     //         .gauge_style(
//     //             Style::default()
//     //                 .fg(Color::White)
//     //                 .bg(Color::Black)
//     //                 .add_modifier(Modifier::ITALIC),
//     //         )
//     //         .percent(percent),
//     //     frame.size(),
//     // );
// }
