use crate::types::Book;
use anyhow::Result;
use cow_utils::CowUtils;
use once_cell::sync::Lazy;
use regex::Regex;

static MULTI_WS_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"[ \t]+").expect("BUG: Invalid regex"));

pub fn run_fixers(book: &mut Book, _dry_run: bool) -> Result<bool> {
    let new_title = fix_title(book.title.clone());
    let new_artist = fix_author(book.author.clone());
    let mut changed = false;

    if let Some(_new_artist) = new_artist {
        changed = true;
        // tags.set_artist(&new_artist);
    }
    if let Some(_new_title) = new_title {
        changed = true;
        // tags.set_title(&new_title);
    }

    // if !dry_run && changed {
    //     tags.write_to_path(&book.path, Version::Id3v24)?;
    // }

    Ok(changed)
}

fn normalise_field(field: String) -> String {
    let new_field = MULTI_WS_RE.replace_all(&field, " ");
    return new_field
        .trim()
        .to_owned()
        .cow_replace('[', "(")
        .cow_replace(']', ")")
        .cow_replace('…', "...")
        .cow_replace('“', "\"")
        .cow_replace('”', "\"")
        .cow_replace('‘', "'")
        .cow_replace('’', "'")
        .to_string();
}

fn fix_author(old_author: Option<String>) -> Option<String> {
    let field = normalise_field(old_author.unwrap_or_default());
    Some(field)
}

fn fix_title(old_title: Option<String>) -> Option<String> {
    let field = normalise_field(old_title.unwrap_or_default());
    Some(field)
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn test_fix_author_ok() {
    //     let given = "Foo Bar".to_string();
    //     let expected = None;
    //     assert_eq!(fix_author(Some(given)), expected);
    // }

    // #[test]
    // fn test_fix_author_multiple() {
    //     let given = "Foo Bar, Baz Qux".to_string();
    //     let expected = "Foo Bar".to_string();
    //     assert_eq!(fix_author(Some(given)), Some(expected));
    // }

    #[test]
    fn test_fix_title_with_quotes() {
        let given = "Foo ‘Bar’ “Wabble”".to_string();
        let expected = "Foo 'Bar' \"Wabble\"".to_string();
        assert_eq!(fix_title(Some(given)), Some(expected));
    }

    #[test]
    fn test_fix_whitespace() {
        let given = "    Foo Bar    ".to_string();
        let expected = "Foo Bar".to_string();
        assert_eq!(fix_title(Some(given)), Some(expected));
    }
}
