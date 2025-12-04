use once_cell::sync::Lazy;
use regex::Regex;
use crate::error::BotError;
use crate::models::{MangaDetail, MangaInfo};
use crate::utils;
use scraper::selectable::Selectable;
use scraper::{Html, Selector};

pub async fn parse_rank(url: &str) -> Result<Vec<MangaInfo>, BotError> {
    let scheme = if url.starts_with("https") {
        "https:"
    } else {
        "http:"
    };

    let content = utils::http::fetch(url).await?;

    let html = Html::parse_document(&content);
    let item_box_sel = Selector::parse(".itemBox").unwrap();
    let item_a_sel = Selector::parse(".itemImg a").unwrap();
    let img_sel = Selector::parse("img").unwrap();
    let info_sel = Selector::parse(".itemTxt").unwrap();
    let rank_sel = Selector::parse("div.number").unwrap();
    let txt_item_a_sel = Selector::parse(".txtItme a").unwrap();
    let pd_span_sel = Selector::parse(".txtItme .pd").unwrap();
    let date_sel = Selector::parse(".date").unwrap();

    let mut mangas = Vec::new();
    for div in html.select(&item_box_sel) {
        if let Some(item_elem) = div.select(&item_a_sel).next() {
            let link = item_elem.attr("href").unwrap_or("").to_string();
            let title = item_elem.attr("title").unwrap_or("").to_string();
            let id = utils::extract_num(&link).unwrap_or(0);

            let mut cover = item_elem
                .select(&img_sel)
                .next()
                .and_then(|img| img.attr("src"))
                .unwrap_or("")
                .to_string();
            if !cover.starts_with("http") && !cover.is_empty() {
                cover = format!("{}{}", scheme, cover);
            }

            let rank = div
                .select(&rank_sel)
                .next()
                .map(|e| {
                    e.text()
                        .collect::<String>()
                        .trim()
                        .parse::<i32>()
                        .unwrap_or(0)
                })
                .unwrap_or(0);

            let (author, total, fav, published) =
                if let Some(info_dom) = div.select(&info_sel).next() {
                    let author = info_dom
                        .select(&txt_item_a_sel)
                        .next()
                        .map(|a| a.text().collect::<String>().trim().to_string())
                        .unwrap_or_default()
                        .trim()
                        .to_string();

                    let mut total = 0;
                    let mut fav = 0;
                    for t in info_dom
                        .select(&pd_span_sel)
                        .map(|s| s.text().collect::<String>())
                    {
                        if t.contains("共") {
                            total = utils::digits_to_i32(&t.trim());
                        } else {
                            fav = utils::digits_to_i32(&t.trim());
                        }
                    }

                    let published = info_dom
                        .select(&date_sel)
                        .next()
                        .map(|e| e.text().collect::<String>().trim().to_string())
                        .unwrap_or_default();

                    (author, total, fav, published)
                } else {
                    (String::new(), 0, 0, String::new())
                };

            mangas.push(MangaInfo {
                id,
                rank,
                title,
                cover,
                author,
                total,
                fav,
                published,
            });
        }
    }
    Ok(mangas)
}

pub async fn parse_detail(id: i64, url: &str) -> Result<MangaDetail, BotError> {
    let scheme = if url.starts_with("https") {
        "https:"
    } else {
        "http:"
    };

    let content = utils::http::fetch(url).await?;

    let html = Html::parse_document(&content);

    let intro_sel = Selector::parse(".Introduct_Sub").unwrap();
    let cover_sel = Selector::parse("img").unwrap();
    let author_sel = Selector::parse("a.introName").unwrap();
    let category_sel = Selector::parse("div.sub_r > p:nth-child(2) > a").unwrap();
    let total_sel = Selector::parse("span.date").unwrap();

    let desc_sel = Selector::parse(".txtDesc").unwrap();

    let (cover, title, author, category, total) = if let Some(intro_elem) = html.select(&intro_sel).next() {
        let (mut cover, title) = if let Some(cover_elem) = intro_elem.select(&cover_sel).next() {
            (cover_elem.attr("src").unwrap_or("").to_string(),
            cover_elem.attr("title").unwrap_or("").to_string())
        } else {
            (String::new(), String::new())
        };
        if !cover.starts_with("http") && !cover.is_empty() {
            if cover.starts_with("////") {
                cover = cover.strip_prefix("//").unwrap_or(&cover).to_string();
            }
            cover = format!("{}{}", scheme, cover);
        }

        let author = if let Some(author_elem) = intro_elem.select(&author_sel).next() {
            author_elem.text().collect::<String>().trim().to_string()
        } else {
            String::new()
        };

        let category = if let Some(category_elem) = intro_elem.select(&category_sel).next() {
            category_elem.text().collect::<String>().trim().to_string()
        } else {
            String::new()
        };
        let total = if let Some(total_elem) = intro_elem.select(&total_sel).next() {
            utils::digits_to_i32(total_elem.text().collect::<String>().trim())
        } else {
            0
        };

        (cover, title, author, category, total)
    } else {
        (String::new(), String::new(), String::new(), String::new(), 0)
    };

    let desc_elem = html.select(&desc_sel).next().unwrap();
    let tag_sel = Selector::parse("a.tagshow").unwrap();

    let mut tags: Vec<String> = Vec::new();
    for tag in desc_elem.select(&tag_sel) {
        tags.push(tag.text().collect::<String>().trim().to_string());
    }

    let description = utils::dom::text_without_any(&desc_elem, &[tag_sel])
        .join("\n")
        .trim()
        .to_string();

    Ok(MangaDetail {
        id,
        title,
        cover,
        author,
        total,
        category,
        tags,
        description,
    })
}

static IMAGE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#"url:\s*fast_img_host\s*\+\s*\\"([^\\"]+)\\""#).unwrap());
pub async fn extract_image_urls(url: &str, base_url: &str) -> Result<Vec<String>, BotError> {
    let content = utils::http::fetch(url).await?;

    let mut images: Vec<String> = Vec::new();
    for cap in IMAGE_RE.captures_iter(&content) {
        if let Some(img) = cap.get(1) {
            images.push(utils::http::resolve_url(img.as_str(), base_url));
        }
    }

    Ok(images)
}
