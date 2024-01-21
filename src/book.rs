use crate::types::Book;
use anyhow::Result;
use epub::doc::EpubDoc;
use std::path::PathBuf;

pub fn get_book(path: PathBuf) -> Result<Book> {
    let doc = EpubDoc::new(path.clone())?;
    let author = doc.mdata("creator");
    let title = doc.mdata("title");
    Ok(Book {
        path,
        author,
        title,
    })
}
