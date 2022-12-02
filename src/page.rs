use std::slice::Iter;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PageMeta {
    pub page: u32,
    pub per_page: u8,
    pub total_count: u32,
    pub total_pages: u32,
}

#[derive(Debug, Deserialize)]
pub struct Page<T> {
    pub meta: PageMeta,
    pub data: Vec<T>,
}

impl<T> IntoIterator for Page<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

impl<'iter, T> IntoIterator for &'iter Page<T> {
    type Item = &'iter T;
    type IntoIter = Iter<'iter, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.iter()
    }
}
