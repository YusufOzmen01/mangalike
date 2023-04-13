use crate::structs::{Scraper, SearchResult};
use color_eyre::Result;
use scraper::Selector;
use crate::selectors::{CHAPTER_IMAGES_PANEL, CHAPTERS, CHAPTERS_LIST, CHAPTERS_PANEL, IMG, RESULTS, RESULTS_PANEL, URL};

impl Scraper {
    pub fn search_manga(&self, query: String) -> Result<Option<Vec<SearchResult>>> {
        let data = reqwest::blocking::get(format!("https://chapmanganato.com/search/story/{}", query.replace(' ', "_")))?.text()?;

        let document = scraper::Html::parse_document(&data);
        let results_panel_selector = Selector::parse(RESULTS_PANEL).unwrap();
        let results_selector = Selector::parse(RESULTS).unwrap();
        let url_selector = Selector::parse(URL).unwrap();
        let img_selector = Selector::parse(IMG).unwrap();

        let results_panel = document.select(&results_panel_selector).collect::<Vec<_>>();
        if results_panel.is_empty() {
            return Ok(None)
        }

        let results = results_panel.get(0).unwrap().select(&results_selector).collect::<Vec<_>>();
        if results.is_empty() {
            return Ok(None)
        }

        let mut mangas: Vec<SearchResult> = Vec::new();

        for result in results {
            let element = result.select(&url_selector).collect::<Vec<_>>();

            let url = element.get(0).unwrap();

            let title = url.select(&img_selector).collect::<Vec<_>>().get(0).unwrap().value().attr("alt").unwrap();
            let url = url.value().attr("href").unwrap().split("manga-").collect::<Vec<&str>>();

            mangas.push(SearchResult {
                title: title.to_string(),
                id: url.get(1).unwrap().to_string(),
            });
        }

        Ok(Some(mangas))
    }

    pub fn get_chapters(&self, manga_id: String) -> Result<Option<Vec<SearchResult>>> {
        let data = reqwest::blocking::get(format!("https://chapmanganato.com/manga-{}", manga_id))?.text()?;

        let document = scraper::Html::parse_document(&data);
        let chapter_panel_selector = Selector::parse(CHAPTERS_PANEL).unwrap();
        let chapter_list_selector = Selector::parse(CHAPTERS_LIST).unwrap();
        let chapters_selector = Selector::parse(CHAPTERS).unwrap();
        let url_selector = Selector::parse(URL).unwrap();

        let chapters_panel = document.select(&chapter_panel_selector).collect::<Vec<_>>();
        if chapters_panel.is_empty() {
            return Ok(None)
        }

        let chapters_list = chapters_panel.get(0).unwrap().select(&chapter_list_selector).collect::<Vec<_>>();
        if chapters_list.is_empty() {
            return Ok(None)
        }

        let chapters = chapters_list.get(0).unwrap().select(&chapters_selector).collect::<Vec<_>>();
        if chapters.is_empty() {
            return Ok(None)
        }

        let mut chapter_vec: Vec<SearchResult> = Vec::new();

        for result in chapters {
            let element = result.select(&url_selector).collect::<Vec<_>>();

            let title = element.get(0).unwrap().value().attr("title").unwrap();
            let url = element.get(0).unwrap().value().attr("href").unwrap().split("chapter-").collect::<Vec<&str>>();

            chapter_vec.push(SearchResult {
                title: title.to_string(),
                id: url.get(1).unwrap().to_string(),
            });
        }

        Ok(Some(chapter_vec))
    }

    pub fn get_chapter_images(&self, manga_id: String, chapter_id: String) -> Result<Option<Vec<String>>> {
        let data = reqwest::blocking::get(format!("https://chapmanganato.com/manga-{}/chapter-{}", manga_id, chapter_id))?.text()?;

        let document = scraper::Html::parse_document(&data);
        let chapter_images_panel_selector = Selector::parse(CHAPTER_IMAGES_PANEL).unwrap();
        let chapter_image_selector = Selector::parse(IMG).unwrap();

        let chapter_images_panel = document.select(&chapter_images_panel_selector).collect::<Vec<_>>();
        if chapter_images_panel.is_empty() {
            return Ok(None)
        }

        let chapter_images = chapter_images_panel.get(0).unwrap().select(&chapter_image_selector).collect::<Vec<_>>();
        if chapter_images.is_empty() {
            return Ok(None)
        }

        let mut chapter_image_links: Vec<String> = Vec::new();

        for element in chapter_images {
            chapter_image_links.push(element.value().attr("src").unwrap().to_string());
        }

        Ok(Some(chapter_image_links))
    }
}