#[derive(Debug, Clone)]
pub struct MangaInfo {
    pub id: i64,
    pub rank: i32,
    pub title: String,
    pub cover: String,
    pub author: String,
    pub total: i32,
    pub fav: i32,
    pub published: String,
}

#[derive(Debug, Clone)]
pub struct MangaDetail {
    #[allow(dead_code)]
    pub id: i64,
    pub title: String,
    pub cover: String,
    pub author: String,
    pub total: i32,
    pub category: String,
    pub tags: Vec<String>,
    pub description: String,
}
