#[derive(serde::Serialize)]
pub enum Direction {
    #[serde(rename = "asc")]
    Ascending,
    #[serde(rename = "desc")]
    Descending,
}
