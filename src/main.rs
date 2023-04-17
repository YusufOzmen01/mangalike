use std::env::current_dir;
use std::fs::File;
use std::{fs, io, thread};
use std::io::Write;
use std::ops::Add;
use std::path::Path;
use std::sync::Arc;
use clap::{Arg, ArgAction, Command};
use color_eyre::Result;
use epub_builder::{EpubBuilder, EpubContent, ZipLibrary};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use crate::manga_list::{add_manga, get_mangas};
use crate::methods::download_image;
use crate::structs::{Scraper, SearchResult};
use crate::utils::{get_manga_starting_chapter};

mod scraper;
mod manga_list;
mod selectors;
mod structs;
mod utils;
mod methods;

fn main() -> Result<()> {
    let matches = Command::new("mangalike")
        .about("sync your mangas in pdf format")
        .version("0.1")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .author("YusufOzmen01")
        .subcommand(
            Command::new("create")
                .short_flag('c')
                .long_flag("create")
                .about("create a library in current folder")
        )
        .subcommand(
            Command::new("sync")
                .short_flag('s')
                .long_flag("sync")
                .about("synchronize your mangas")
        )
        .subcommand(
            Command::new("export")
                .short_flag('e')
                .long_flag("export")
                .about("export your mangas as epub")
        )
        .subcommand(
            Command::new("query")
                .short_flag('q')
                .long_flag("query")
                .about("query mangas")
                .arg(
                    Arg::new("name")
                        .short('n')
                        .long("name")
                        .help("name of the manga you wanna search")
                        .required(true)
                        .action(ArgAction::Set)
                )
        ).get_matches();

    let scraper = Scraper::default();

    match matches.subcommand() {
        Some(("create", _)) => {
            let current_dir = current_dir()?;
            let manga_list_file = Path::new(&current_dir).join("mangalike.toml");

            if manga_list_file.exists() {
                println!("Library already exists, skipping action");

                return Ok(());
            }

            File::create(manga_list_file)?;

            println!("Library created in {}", current_dir.to_str().unwrap())
        }

        Some(("export", _)) => {
            let current_dir = current_dir()?;
            let manga_list_file = Path::new(&current_dir).join("mangalike.toml");

            if !manga_list_file.exists() {
                println!("This folder does not contain a library! Create one with --create");

                return Ok(());
            }

            let mangas = get_mangas(&current_dir)?;
            if let Some(mangas) = mangas {
                let m = MultiProgress::new();

                let stopped = ProgressStyle::with_template(
                    "{msg} [{elapsed_precise}] {bar:40.burgundy/red} {pos:>7}/{len:7}",
                ).unwrap().progress_chars("▮-");

                let started = ProgressStyle::with_template(
                    "{msg} [{elapsed_precise}] {bar:40.green/green} {pos:>7}/{len:7}",
                ).unwrap().progress_chars("▮-");

                let manga_pb = m.add(ProgressBar::new(mangas.len() as u64));

                manga_pb.set_style(started.clone());
                m.println("Starting export...")?;

                for manga in mangas {
                    let manga_folder = Path::new(&current_dir).join(format!("{} - {}", &manga.id, &manga.title));
                    let chapter_order = fs::read_to_string(Path::new(&manga_folder).join("order.txt"))?;
                    let chapters = chapter_order.split('\n');

                    let mut epub =EpubBuilder::new(ZipLibrary::new().unwrap()).unwrap();
                    let mut builder = epub.metadata("title", manga.clone().title).unwrap();

                    let chapter_pb = m.insert_after(&manga_pb, ProgressBar::new(chapters.clone().count() as u64));
                    chapter_pb.set_style(stopped.clone());

                    chapter_pb.inc(1);

                    for (i, chapter) in chapters.clone().enumerate() {
                        let chapter_folder = Path::new(&manga_folder).join(chapter);
                        if !chapter_folder.exists() {
                            println!("Reached the end of chapters");

                            break;
                        }

                        let mut body = String::new();

                        for j in 0.. {
                            let image_path = Path::new(&chapter_folder).join(format!("{}.jpg", j));
                            if !image_path.exists() {
                                break;
                            }

                            let image = File::open(image_path)?;

                            builder = builder.add_resource(format!("manga-{}-chapter-{}-{}.jpg", manga.clone().id, i, j), image, "image/jpeg").unwrap();
                            body = body.add(&*format!("<img src=\"manga-{}-chapter-{}-{}.jpg\"></img><br>", manga.clone().id, i, j));
                        }

                        if chapter.trim().is_empty() {
                            continue;
                        }

                        builder = builder.add_content(
                            EpubContent::new(format!("chapter-{}.xhtml", i), body.as_bytes())
                                .title(format!("Chapter {}", chapter))
                        ).unwrap();

                        chapter_pb.set_style(started.clone());
                        let mut message = format!("Chapter: [{}]", chapter);
                        for _ in 0..30 - message.len() as u32 {
                            message += " ";
                        }
                        chapter_pb.set_message(message);
                        chapter_pb.inc(1);
                    }

                    let exports = Path::new(&current_dir).join("exports");
                    if !exports.exists() {
                        fs::create_dir(&exports)?;
                    }

                    let file_path = exports.join(format!("{}.epub", manga.clone().title));
                    if file_path.exists() {
                        fs::remove_file(&file_path)?;
                    }

                    m.println("Generating epub...")?;
                    let file = File::create(file_path)?;
                    builder.generate(file).unwrap();
                    m.println("Epub generated!")?;

                    let mut message = format!("Manga: [{}]", manga.id);
                    for _ in 0..30 - message.len() as u32 {
                        message += " ";
                    }
                    manga_pb.set_message(message);
                    manga_pb.inc(1);
                }

                m.clear()?;
            }

            println!("All mangas exported as epub!");

            return Ok(());
        }

        Some(("sync", _)) => {
            let current_dir = current_dir()?;
            let manga_list_file = Path::new(&current_dir).join("mangalike.toml");

            if !manga_list_file.exists() {
                println!("This folder does not contain a library! Create one with --create");

                return Ok(());
            }

            let client = Arc::new(reqwest::blocking::Client::new());

            let mangas = get_mangas(&current_dir)?;
            if let Some(mangas) = mangas {
                let m = MultiProgress::new();

                let manga_pb = Arc::new(m.add(ProgressBar::new(mangas.len() as u64)));

                let style = ProgressStyle::with_template(
                    "{msg} [{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7}",
                ).unwrap().progress_chars("##-");

                manga_pb.set_style(style.clone());
                let mut message = String::from("Manga [Checking]:");
                for _ in 0..30 - message.len() as u32 {
                    message += " ";
                }
                manga_pb.set_message(message);

                m.println("Starting synchronization...")?;
                manga_pb.inc(1);

                for manga in mangas {
                    let chapters = scraper.get_chapters(manga.clone().id.trim().to_string())?;

                    if let Some(chapters) = chapters {
                        let chapter_pb = Arc::new(m.insert_after(&manga_pb, ProgressBar::new(chapters.len() as u64)));
                        chapter_pb.set_style(style.clone());

                        chapter_pb.inc(1);

                        let mut chapters = chapters.into_iter().rev().collect::<Vec<SearchResult>>();
                        let manga_folder = Path::new(&current_dir).join(format!("{} - {}", &manga.id, &manga.title));

                        let mut starting = 0;

                        for (i, chapter) in chapters.clone().iter().enumerate() {
                            if let Ok(Some(starting_chapter)) = get_manga_starting_chapter(manga.id.clone(), current_dir.to_str().unwrap().to_string()) {
                                if chapter.id == starting_chapter {
                                    starting = i as i32;
                                }
                            }
                        }

                        chapters.drain(0..starting as usize);

                        if let Ok(mut order) = File::create(Path::new(&manga_folder).join("order.txt")) {
                            for chapter in chapters.clone() {
                                let line = chapter.id + "\n";
                                order.write_all(line.as_bytes())?;
                            }
                        }

                        for chapter in chapters {
                            let chapter_folder = Path::new(&current_dir).join(format!("{} - {}", &manga.id, &manga.title)).join(chapter.clone().id);

                            if !chapter_folder.exists() {
                                fs::create_dir(&chapter_folder)?;
                            }

                            let images = scraper.get_chapter_images(manga.clone().id, chapter.clone().id)?;

                            if let Some(images) = images {
                                let image_pb = Arc::new(m.insert_after(&chapter_pb, ProgressBar::new(images.len() as u64)));
                                image_pb.set_style(style.clone());

                                let mut threads = vec![];

                                for (i, image) in images.iter().enumerate() {
                                    let image_path = Path::new(&chapter_folder.clone()).join(format!("{}.jpg", i));
                                    let manga_pb = manga_pb.clone();
                                    let chapter_pb = chapter_pb.clone();
                                    let image_pb = image_pb.clone();
                                    let image = image.to_string();
                                    let manga_id = manga.id.clone();
                                    let chapter_id = chapter.id.clone();
                                    let client = client.clone();

                                    threads.push(thread::spawn(move || download_image(
                                        image_path,
                                        vec![manga_pb, chapter_pb, image_pb].as_slice(),
                                        image,
                                        manga_id,
                                        chapter_id,
                                        client
                                    )));
                                }

                                for t in threads {
                                    t.join().unwrap()?;
                                }
                            }

                            let mut message = format!("Chapter [{}]:", chapter.id);
                            for _ in 0..30 - message.len() as u32 {
                                message += " ";
                            }
                            chapter_pb.set_message(message);
                            chapter_pb.inc(1);
                        }
                    }

                    let mut message = format!("Manga [{}]:", manga.id);
                    for _ in 0..30 - message.len() as u32 {
                        message += " ";
                    }
                    manga_pb.set_message(message);
                    manga_pb.inc(1);
                }

                m.clear()?;
            }

            println!("Library synced successfully!");

            return Ok(());
        }

        Some(("query", query)) => {
            let current_dir = current_dir()?;
            let manga_list_file = Path::new(&current_dir).join("mangalike.toml");

            if !manga_list_file.exists() {
                println!("This folder does not contain a library! Create one with --create");

                return Ok(());
            }

            let query_vec = query.get_many::<String>("name").unwrap().collect::<Vec<&String>>();
            let query_str = query_vec.get(0).unwrap().to_owned().to_owned();

            println!("Searching...");
            let results = scraper.search_manga(query_str)?;

            if results.is_none() {
                println!("No results found.");

                return Ok(());
            }

            let results = results.unwrap();

            for (i, result) in results.iter().enumerate() {
                println!("[{}]: {}", i+1, result.title)
            }

            let stdin = io::stdin();

            loop {
                println!("Which one do you want to add?: ");

                let mut input = String::new();
                stdin.read_line(&mut input)?;

                let index: i32 = input.trim().parse()?;
                if index >= results.len() as i32 && index - 1 < 0 {
                    println!("Invalid index.");

                    continue;
                }

                let selected = results.get((index-1) as usize).unwrap();
                add_manga(selected.clone().id, selected.clone().title, current_dir.to_str().unwrap().to_string())?;

                println!("{} added to library successfully! To delete it, you can delete its folder.", selected.title);

                return Ok(());
            }
        }
        _ => unreachable!()
    }


    Ok(())
}
