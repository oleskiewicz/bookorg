use crate::types::Book;
use anyhow::Result;
use id3::Tag;
use std::path::PathBuf;

pub fn get_book(path: PathBuf) -> Result<Book> {
    let tag = Tag::read_from_path(&path)?;
    Ok(Book { path, tag })
}
