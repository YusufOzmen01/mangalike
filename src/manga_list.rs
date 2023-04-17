#[allow(deprecated)]

use std::fs;
use std::path::{Path, PathBuf};
use color_eyre::Result;
use crate::structs::SearchResult;

pub fn add_manga(manga_id: String, manga_name: String, cwd: String) -> Result<()> {
    if Path::new(&cwd).join(format!("{} - {}", &manga_id, &manga_name)).exists() {
        return Ok(());
    }

    fs::create_dir(Path::new(&cwd).join(format!("{} - {}", &manga_id, &manga_name)))?;

    Ok(())
}

pub fn get_mangas(cwd: &PathBuf) -> Result<Option<Vec<SearchResult>>> {
    let folders = fs::read_dir(Path::new(&cwd))?;

    let mut result: Vec<SearchResult> = Vec::new();

    for folder in folders {
        let folder = folder.unwrap();

        if !folder.file_type().unwrap().is_dir() {
            continue;
        }

        let name = folder.file_name();
        let name = name.to_str().unwrap();

        if !name.contains(" - ") {
            continue;
        }

        let folder_name = name.splitn(2, " - ").collect::<Vec<&str>>();

        if folder_name.len() != 2 {
            continue;
        }

        let id = folder_name.first().unwrap().to_string();
        let name = folder_name.get(1).unwrap().to_string();

        result.push(SearchResult {
            id,
            title: name
        });
    }

    Ok(Some(result))
}