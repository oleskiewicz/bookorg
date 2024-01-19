use clap::Parser;
use id3::Tag;
use std::path::PathBuf;

pub struct Book {
    pub path: PathBuf,
    pub tag: Tag,
}

#[derive(Debug, PartialEq, Eq)]
pub struct TrackFeat {
    pub title: String,
    pub featured_artists: Vec<String>,
    pub original_title: String,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    #[arg(
        long,
        short = 'n',
        help = "Don't actually rename or tag files, only display what would happen"
    )]
    pub dry_run: bool,

    #[arg(
        long,
        short,
        help = "Ignore .lastbookorg timestamp, run on all files present regardless"
    )]
    pub force: bool,

    #[arg(
        long,
        short,
        help = "Use a different output directory (by default, it's the same as the input dir)"
    )]
    pub output_dir: Option<PathBuf>,

    /// The format to apply to files, excluding the extension.
    ///
    /// Substitutions can be applied inside curly brackets, for example with {artist} to get the
    /// track artist. Any formats returning data with "/" will have it transformed to "_".
    ///
    /// Available formats:
    ///
    /// TAG:
    ///
    ///   artist
    ///   album
    ///   track  (width: 2)
    ///   title
    ///
    /// LITERAL:
    ///
    ///   {{ and }} indicate literal brackets.
    #[arg(
        long,
        verbatim_doc_comment,
        default_value = "{artist}/{album}/{track} {title}"
    )]
    pub fmt: String,

    #[arg(help = "Directories to find ebook files in.")]
    pub paths: Option<Vec<PathBuf>>,
}
