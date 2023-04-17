use std::fs::File;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;
use color_eyre::Result;
use indicatif::{ProgressBar};
use reqwest::blocking::Client;
use reqwest::header::REFERER;

pub fn download_image(
    image_path: PathBuf,
    progress_bars: &[Arc<ProgressBar>],
    image: String,
    manga_id: String,
    chapter_id: String,
    client: Arc<Client>
) -> Result<()> {
    loop {
        if image_path.exists() {
            let mut message = format!("Manga [{} Checking]:", manga_id);
            for _ in 0..30 - message.len() as u32 {
                message += " ";
            }

            progress_bars[0].set_message(message);

            let mut message = format!("Chapter [{} Checking]:", chapter_id);
            for _ in 0..30 - message.len() as u32 {
                message += " ";
            }
            progress_bars[1].set_message(message);

            progress_bars[2].inc(1);

            break;
        }

        let resp = client.get(image.clone()).header(REFERER, "https://chapmanganato.com/").send();
        if let Ok(mut resp) = resp {
            let mut out = File::create(&image_path)?;

            io::copy(&mut resp, &mut out)?;

            let mut message = format!("Manga [{}]:", manga_id);
            for _ in 0..30 - message.len() as u32 {
                message += " ";
            }
            progress_bars[0].set_message(message);

            let mut message = format!("Chapter [{}]:", chapter_id);
            for _ in 0..30 - message.len() as u32 {
                message += " ";
            }
            progress_bars[1].set_message(message);

            let mut message = String::from("Image:");
            for _ in 0..30 - message.len() as u32 {
                message += " ";
            }
            progress_bars[2].set_message(message);
            progress_bars[2].inc(1);

            break;
        }
    }

    Ok(())
}