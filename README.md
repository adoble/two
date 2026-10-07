# two

`two` converts a single-file TiddlyWiki 5 HTML export into files for an
[Obsidian](https://obsidian.md/) vault. WikiText tiddlers become Markdown notes;
supported Markdown tiddlers and image tiddlers are also written to the output
directory.

## Why use two?

- Convert supported user tiddlers in one export, or select a single tiddler by title.
- Translate common WikiText formatting, headings, lists, tables, links, images,
  and transclusions into Markdown and Obsidian link syntax.
- Preserve tiddler tags as Obsidian tags.
- Copy existing Markdown and supported image files (JPEG, PNG, GIF, and SVG).
- Skip TiddlyWiki system tiddlers whose titles begin with `$:/`.

## Get started

### Requirements

- Rust and Cargo (stable Rust 1.85 or later; the project uses the 2024 edition).
- A single-file TiddlyWiki 5 HTML export containing its embedded tiddler store.

### Build

Clone the repository and build the optimized executable:

```sh
git clone https://github.com/adoble/two.git
cd two
cargo build --release --locked
```

The executable will be available at `target/release/two`.

### Convert a wiki

Pass the HTML export and the destination vault directory:

```sh
./target/release/two /path/to/wiki.html --output /path/to/obsidian-vault
```

To convert only one tiddler, pass its exact title with `--tiddler`:

```sh
./target/release/two /path/to/wiki.html \
  --output /path/to/obsidian-vault \
  --tiddler "RustResources"
```

The output directory is created if it does not already exist. Use `-o` for
`--output`, `-t` for `--tiddler`, and `--help` to see all command-line options.

## Development

Run the test suite with:

```sh
cargo test
```
