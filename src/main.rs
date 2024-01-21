mod book;
mod fixers;
mod mtime;
mod rename;
mod types;

use anyhow::Result;
use clap::Parser;
use funcfmt::{fm, FormatMap, FormatPieces, ToFormatPieces};
use jwalk::WalkDir;
use rayon::prelude::*;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

const ALLOWED_EXT: &str = "epub";

fn fix_book(book: &mut types::Book, dry_run: bool) {
    let fix_results = fixers::run_fixers(book, dry_run);
    match fix_results {
        Ok(_applied_fixers) => {
            // if applied_fixers {
            //     print_updated_tags(book);
            // }
        }
        Err(err) => eprintln!("cannot fix {}: {:?}", book.path.display(), err),
    }
}

// fn print_updated_tags(book: &types::Book) {
//     println!(
//         "{}: updated tags: author: '{}', title: '{}'",
//         book.path.display(),
//         book.author.unwrap_or_default(),
//         book.title.unwrap_or_default(),
//     );
// }

fn rename_book(
    track: &types::Book,
    fp: &FormatPieces<types::Book>,
    output_path: &Path,
    dry_run: bool,
) {
    let new_path = rename::rename_item(track, fp, output_path, dry_run);

    match new_path {
        Ok(Some(new_path)) => println!("{} -> {}", track.path.display(), new_path.display()),
        Ok(None) => (),
        Err(err) => eprintln!("cannot rename {}: {:?}", track.path.display(), err),
    }
}

const ADDITIONAL_ACCEPTED_CHARS: &[char] = &['.', '-', '(', ')', ','];

fn clean_part(path_part: String) -> String {
    path_part
        .chars()
        .map(|c| {
            if c.is_alphanumeric()
                || c.is_whitespace()
                || ADDITIONAL_ACCEPTED_CHARS.iter().any(|&a| a == c)
            {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn get_format_pieces(tmpl: &str) -> Result<funcfmt::FormatPieces<types::Book>> {
    let formatters: FormatMap<types::Book> = fm!(
        "author" => |t: &types::Book| Some(clean_part(
            t.author.clone().unwrap_or("Unknown".to_string())
        )),
        "title" => |t: &types::Book| Some(clean_part(
            t.title.clone().unwrap_or("Untitled".to_string())
        )),
    );

    Ok(formatters.to_format_pieces(tmpl)?)
}

fn is_updated_since_last_run(path: &PathBuf, last_run_time: SystemTime) -> bool {
    mtime::mtime_def_now(path) > last_run_time
}

fn fix_all_books(cfg: &types::Config, base_path: &PathBuf, output_path: &Path) {
    // If the output path is different, we don't know if we should run or not, so just do them all
    let last_run_time = if output_path == base_path {
        mtime::get_last_run_time(base_path).unwrap_or(SystemTime::UNIX_EPOCH)
    } else {
        SystemTime::UNIX_EPOCH
    };

    let fp = match get_format_pieces(&cfg.fmt) {
        Ok(fp) => fp,
        Err(err) => {
            eprintln!("fatal: {err}");
            std::panic::set_hook(Box::new(|_| {}));
            panic!(); // Don't use exit() because it does not run destructors
        }
    };

    WalkDir::new(base_path)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path())
        .filter(|e| {
            let ext = e
                .extension()
                .and_then(OsStr::to_str)
                .unwrap_or("")
                .to_lowercase();
            ALLOWED_EXT == ext
        })
        .filter(|e| cfg.force || is_updated_since_last_run(e, last_run_time))
        .collect::<Vec<_>>()
        .into_par_iter()
        .for_each(|path| match book::get_book(path.clone()) {
            Ok(mut i) => {
                fix_book(&mut i, cfg.dry_run);
                rename_book(&i, &fp, output_path, cfg.dry_run);
            }
            Err(err) => eprintln!("error: {}: {err:?}", path.display()),
        });

    if !cfg.dry_run && output_path == base_path {
        mtime::set_last_run_time(base_path).unwrap_or_else(|err| {
            eprintln!(
                "can't set last run time for {}: {:?}",
                base_path.display(),
                err
            );
        });
    }
}

fn main() {
    let mut cfg = types::Config::parse();

    let paths = match cfg.paths.take() {
        Some(paths) => paths,
        None => vec![PathBuf::from(".")],
    };

    for path in paths {
        let this_output_path;

        if let Some(op) = cfg.output_dir.clone() {
            this_output_path = op;
        } else {
            this_output_path = path.clone();
        }

        fix_all_books(&cfg, &path, &this_output_path);
    }
}
