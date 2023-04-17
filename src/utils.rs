use std::fs;
use std::ops::Add;
use std::path::Path;
use color_eyre::Result;

pub fn get_manga_starting_chapter(manga_id: String, cwd: String) -> Result<Option<String>> {
    let data = fs::read(Path::new(&cwd).join("mangalike.toml"))?;
    let data = String::from_utf8(data)?;
    let data = data.split('\n').collect::<Vec<_>>();

    for (i, line) in data.iter().enumerate() {
        let parts = line.splitn(2, '=').collect::<Vec<_>>();

        if parts.len() != 2 {
            return Ok(None);
        }

        if *parts.first().unwrap() == manga_id {
            return Ok(Some(parts.get(1).unwrap().to_string()))
        }
    }

    Ok(None)
}