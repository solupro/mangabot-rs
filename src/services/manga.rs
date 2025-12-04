use crate::error::BotError;
use crate::models::MangaInfo;
use crate::utils;
use scraper::selectable::Selectable;
use scraper::{Html, Selector};
use crate::utils::extract_num;

fn digits_to_i32(s: &str) -> i32 {
    s.chars()
        .filter(|c| c.is_ascii_digit())
        .collect::<String>()
        .parse::<i32>()
        .unwrap_or(0)
}

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
                            total = digits_to_i32(&t.trim());
                        } else {
                            fav = digits_to_i32(&t.trim());
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
