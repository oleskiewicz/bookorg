use crate::extract::extract_feat;
use crate::types::{Book, TrackFeat};
use anyhow::Result;
use cow_utils::CowUtils;
use id3::{TagLike, Version};
use once_cell::sync::Lazy;
use regex::Regex;

static MULTI_WS_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"[ \t]+").expect("BUG: Invalid regex"));

pub fn run_fixers(track: &mut Book, dry_run: bool) -> Result<bool> {
    let tags = &mut track.tag;

    let new_title = fix_title(tags.title(), tags.artist());
    let new_artist = fix_author(tags.artist());
    let mut changed = false;

    if let Some(new_artist) = new_artist {
        changed = true;
        tags.set_artist(&new_artist);
    }
    if let Some(new_title) = new_title {
        changed = true;
        tags.set_title(&new_title);
    }

    if !dry_run && changed {
        tags.write_to_path(&track.path, Version::Id3v24)?;
    }

    Ok(changed)
}

fn normalise_field(field: &str) -> String {
    let mut new_field = field.to_owned();

    new_field = MULTI_WS_RE.replace_all(&new_field, " ").to_string();
    new_field = new_field.trim().to_owned();
    new_field
        .cow_replace('[', "(")
        .cow_replace(']', ")")
        .cow_replace('…', "...")
        .cow_replace('“', "\"")
        .cow_replace('”', "\"")
        .cow_replace('‘', "'")
        .cow_replace('’', "'")
        .to_string()
}

fn fix_author(old_author: Option<&str>) -> Option<String> {
    let field = normalise_field(old_author.unwrap_or_default());
    let artist = extract_feat(&field);
    if artist.title == artist.original_title {
        None
    } else {
        Some(artist.title)
    }
}

fn fix_title(old_title: Option<&str>, old_author: Option<&str>) -> Option<String> {
    let Some(old_title) = old_title else {
        return None;
    };
    let old_title = extract_feat(old_title);
    let old_artist = extract_feat(old_author.unwrap_or_default());
    let new_title = make_title(&old_title, old_artist);

    if new_title == old_title.original_title {
        None
    } else {
        Some(new_title)
    }
}

fn make_title(title: &TrackFeat, author: TrackFeat) -> String {
    let mut featured_artists = title.featured_artists.clone();
    featured_artists.extend(author.featured_artists);

    let mut new_title = title.title.clone();
    if !featured_artists.is_empty() {
        let feat_artists_string = make_feat_string(&featured_artists);
        let feat_string = format!(" (feat. {feat_artists_string})");
        new_title.push_str(&feat_string);
    }

    normalise_field(&new_title)
}

fn make_feat_string(featured_artists: &[String]) -> String {
    let mut output = String::new();
    let mut artist_idx = 1i32;

    let mut artists = featured_artists.iter().peekable();

    while let Some(artist) = artists.next() {
        let is_last = artists.peek().is_none();

        if is_last && artist_idx > 2 {
            output.push_str(", and ");
        } else if is_last && artist_idx == 2 {
            output.push_str(" and ");
        } else if artist_idx != 1 {
            output.push_str(", ");
        }
        output.push_str(artist);
        artist_idx = artist_idx.checked_add(1).expect("overflow");
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fix_artist_no_feat() {
        let given = "Foo Bar";
        let expected = None;
        assert_eq!(fix_author(Some(given)), expected);
    }

    #[test]
    fn test_fix_artist_with_feat() {
        let given = "Foo Bar (feat. Baz Qux)";
        let expected = Some("Foo Bar".to_owned());
        assert_eq!(fix_author(Some(given)), expected);
    }

    #[test]
    fn test_fix_title_no_title_feat_no_artist_feat() {
        let given_title = "Foo Bar";
        let given_artist = "Baz Qux";
        let expected = None;
        assert_eq!(fix_title(Some(given_title), Some(given_artist)), expected);
    }

    #[test]
    fn test_fix_title_with_title_feat_no_artist_feat() {
        let given_title = "Foo Bar (feat. Wibble Wobble)";
        let given_artist = "Baz Qux";
        let expected = None;
        assert_eq!(fix_title(Some(given_title), Some(given_artist)), expected);
    }

    #[test]
    fn test_fix_title_with_title_feat_no_artist_feat_and_brackets() {
        let given_title = "Foo Bar (feat. Wibble Wobble) [Richard Stallman mix]";
        let given_artist = "Baz Qux";
        let expected = Some("Foo Bar (Richard Stallman mix) (feat. Wibble Wobble)".to_owned());
        assert_eq!(fix_title(Some(given_title), Some(given_artist)), expected);
    }

    #[test]
    fn test_fix_title_no_title_feat_with_artist_feat() {
        let given_title = "Foo Bar";
        let given_artist = "Baz Qux feat. Fizz Buzz";
        let expected = Some("Foo Bar (feat. Fizz Buzz)".to_owned());
        assert_eq!(fix_title(Some(given_title), Some(given_artist)), expected);
    }

    #[test]
    fn test_fix_title_with_title_feat_and_artist_feat() {
        let given_title = "Foo Bar (feat. Wibble Wobble)";
        let given_artist = "Baz Qux feat. Fizz Buzz";
        let expected = Some("Foo Bar (feat. Wibble Wobble and Fizz Buzz)".to_owned());
        assert_eq!(fix_title(Some(given_title), Some(given_artist)), expected);
    }

    #[test]
    fn test_fix_title_with_title_feat_smart_quotes() {
        let given_title = "Foo ‘Bar’ (feat. Wibble “Wabble” Wobble)";
        let given_artist = "Baz Qux";
        let expected = Some("Foo 'Bar' (feat. Wibble \"Wabble\" Wobble)".to_owned());
        assert_eq!(fix_title(Some(given_title), Some(given_artist)), expected);
    }

    #[test]
    fn test_fix_whitespace() {
        let given = "    Foo Bar [feat.    Baz    Qux   ]    ";
        let expected = Some("Foo Bar (feat. Baz Qux)".to_owned());
        assert_eq!(fix_title(Some(given), None), expected);
    }
}
