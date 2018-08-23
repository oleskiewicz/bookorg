use taglib;
use regex::Regex;
use types::{MackError, Track, TrackFeat};
use extract::extract_feat;

lazy_static! {
    static ref MULTI_WS_RE: Regex = Regex::new(r#"[ \t]+"#).unwrap();
}

pub fn run_fixers(track: &mut Track, _dry_run: bool) -> Result<bool, MackError> {
    let tags = track.tag_file.tag()?;

    fixer_is_blacklisted(&tags)?;

    let new_title = fix_title(&tags);
    let new_artist = fix_artist(&tags);

    Ok(new_title.is_some() || new_artist.is_some())
}

fn fix_artist(tags: &taglib::Tag) -> Option<String> {
    let artist = extract_feat(tags.artist());
    if artist.title != artist.original_title { Some(artist.title) } else { None }
}

fn fix_title(tags: &taglib::Tag) -> Option<String> {
    let old_title = extract_feat(tags.title());
    let old_artist = extract_feat(tags.artist());

    let new_title = make_title(&old_title, &old_artist);

    if new_title != old_title.original_title { Some(new_title) } else { None }
}

fn make_title(title: &TrackFeat, artist: &TrackFeat) -> String {
    let mut featured_artists = title.featured_artists.clone();
    featured_artists.extend(artist.featured_artists.clone());

    let mut new_title = MULTI_WS_RE.replace_all(&title.title, " ").trim().to_owned();
    if !featured_artists.is_empty() {
        let feat_artists_string = make_feat_string(featured_artists);
        let feat_string = format!(" (feat. {})", feat_artists_string);
        new_title.push_str(&feat_string);
    }

    new_title
}

fn make_feat_string(featured_artists: Vec<String>) -> String {
    let mut output = "".to_owned();
    let mut artist_idx = 1;

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
        artist_idx += 1;
    }

    output
}

fn fixer_is_blacklisted(tags: &taglib::Tag) -> Result<(), MackError> {
    if tags.comment().contains("_NO_MACK") { Err(MackError::Blacklisted) } else { Ok(()) }
}
