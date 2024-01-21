# bookorg

bookorg is an opinionated, fast ebook organiser. It enforces:

- Directory layout
- File name format

It is forked from [mack](https://github.com/cdown/mack), a music organiser.

## Installation

    cargo install bookorg

## Configuration

You can change the default format as follows:

    bookorg --fmt '{author} - {title}'

The default is `{author}/{title}`. The epub extension is added automatically.

## Example usage

Take all books from Library, Documents, and Downloads, move and organise them in the Library:

    bookorg -n --fmt '{author} - {title}' -o ~/Library -- ~/Library ~/Documents ~/Downloads
