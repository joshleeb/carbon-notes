# Carbon Notes

Note taking system and curator.

## Background

I've been keeping track of my notes using markdown files synced with Dropbox.
This provides a lot of benefits, such as

- Plain text note storage
- Local editing in any editor (Vim)
- Completely control over notes structure
- Syncing between multiple devices including mobile

Recently though, there have been a number of other features that I would like as
well, see the [Minimal Features] section below. Most of these features come
stock standard with off the shelf notes managers, such as [Notion], [Dropbox
Paper], and [Boostnote]. But these notes managers also sacrifice some of the
core features I've come to rely on.

## Goals

Implement the minimal features listed below while maintaining the current
features and requiring little maintenance effort.

### Non-Goals

- Building yet another note management system that tries to solve all problems
for all the people
- Storing notes on my own hosted server that supports syncing
- Implementing a web based notes editor

## Functionality

### Minimal Features

- Nicely rendered notes viewable in web browser (Firefox)
    - Preferable with github readme styling
    - Entirely static (no-server)
- Linking between source files
    - Need to be able to check for broken links
    - Show a list of places that are linked to a file, in case the user wants to
    change the path
    - Alternatively have a path-independent linking system (UUID + scan)
- Checking for broken links
- Syntax highlighting of code blocks
- Handles images and other external content
    - Including non-markdown notes
- Automatically generated index pages per directory

### Extra Features

- Rendering of math formula
- Markdown note linting
- Watch mode
    - Re-render files on change
    - Reload the browser as well (Stretch feature which may not be possible with
    static HTML)
- Incremental building
- Note inclusion
    - Inserting notes into other notes
    - Not sure how useful this would be, or how to make it work nicely such that
    it is obvious a note won't be rendered as standalone but as part of another
    note.

### Longer Term Features

- Supporting multiple markup source formats
    - Org Mode, Restructured Text, Creole (Wikitext Markup)

## Design

> Keep in mind [12 Factor CLI Tools][12-factor-cli-tools] when designing the
> `carbon` command structure.

Let's start with the directory structure. I plan to use this with Dropbox to
facilitate syncing between devices, but this should work for any generic UNIX
based file system.

```bash
$ tree notes-dir
notes-dir
├── _rendered
├── _static
├── note1.md
├── note2.md
├── subdir1
│   ├── _static
│   └── note3.md
└── subdir2
    ├── note4.md
    ├── note5.pdf
    └── note6.png
```

We'll go over the `_rendered/` directory a bit further down since it is
generated by the `carbon` tool.

The `_static/` directories hold any static links that are used in our notes. For
the most part this will be images, but could be videos, gifs, or anything that
can be linked to in a markdown page.

The rest of the directory shows a typical notes structure. It's worth noting
that we have a mix of markdown notes with other file formats, such as PDF and
PNG. We want `carbon` to include these notes in our rendered view so that we can
be certain we aren't missing information, or jumping between viewing methods
when consuming our notes.

### Rendered Directory

The `_rendered/` directory is generated by `carbon`. It has a file structure
nearly identical to the `notes-dir/` whereby all markdown notes are converted to
styled HTML, and index pages are created to help navigate through the directory.

```bash
$ tree notes-dir/_rendered/
_rendered
├── note1.html
├── note2.html
├── subdir1
│   ├── index.html
│   └── note3.html
└── subdir2
    ├── index.html
    └── note4.html
```

Notice that the markdown notes have the same name, but are now suffixed with
`html` rather than `md`. 

The PDF and PNG notes also haven't been copied over. This is because these files
may be sufficiently large that we don't want to copy them or there could be many
other files of that type, such as if we were storing a photo album. This
behaviour can be changed to copy the files, and use links in the `_rendered/`
directory with a flag or config value.

To ensure that we can still view these files when rendered, they will be linked
to in the respective `index.html` page which will contain an absolute path for
the note. And similarly, links in the markdown notes to these notes will be
replaced by absolute paths.

This does mean that moving these non-markdown files will result in broken links
if they are used in markdown notes. To help fix this, `carbon` will display a
list of the location of broken links (both inline and reference type) when
rendering or run with 

```bash
$ cd notes-dir && carbon check-links
note1.md:17 blah blah blah [broken-link](./this-link-is-broken) blah blah
subdir1/note3.md:42 [broken-link]: ./this-link-is-broken
```

All paths used in `index.html` files will be absolute links. Additionally, all
links with local paths used in markdown notes will also be converted to absolute
paths.

### Index Pages

Index pages will be generated for each directory in `notes-dir/` excluding
`_rendered/` and `_static/`. These will contain links to each note within the
directory, and links to index pages of subdirectories as well. Additionally,
there will be a link to go up a directory. 

Having generated HTML as index pages allows us to embed JS and other extra
features, such as jumping to a specific path, into the page.

### Rendering

- Markdown to HTML with the [`pulldown-cmark`][pulldown-cmark] crate
- Syntax highlighting with the [`syntect`][syntect] crate
    - Theme can be configured in the config or with a flag
- Math formula rendering with [MathJax] JS embedded into rendered notes
- Styling will be with a CSS stylesheet linked from each rendered markdown note
    - Styling can be configured in the config or with a flag
- Can be highly parallelized.

See the `concept/` directory for a proof of concept for combining these to
render HTML notes from markdown.

### Linting

Linting can be achieved with the [markdownlint] tool.

We may decide to include a subcommand in `carbon` to interface with this tool.

### Watch Mode

Watch mode can be achieved with the [`entr`][entr] tool (or similar).

Depending on how fast rendering will run for a large set of notes, and how easy
it will be at that point to implement incremental rendering, we may include a
watch command in the future.

### Concatenation

It might be useful to be able to include a note within another note so that when
editing they are separate, but when rendered they are a single standalone file.

This is something that should be fairly trivial to implement but does require a
few careful considerations.

- Handling cyclical inclusions.
    - E.g: A includes B; B includes C; C includes A.
- Making it very clear and explicit that a note is included within another note
and so it won't be rendered as a standalone file.
    - Or not and leaving it up to the user to remember that noted can be
    included and so if they can't find a specific note being rendered as a
    standalone file, then that could be the reason.
    - We could render both the standalone notes, and the aggregate note that
    includes a bunch of notes, but I think this would lead to more confusion
    than if we didn't do this.

### Incremental Rendering

This will require more thought when we get up to it, but for now these are some
initial notes.

- Would be good to know what changed
    - File content updated vs directory tree changed (e.g: moved file)
- May need to check links to ensure that none are broken after change
- Could be done on within a directory or a note scope
    - Note would be much faster and useful when editing a note
- Storing file hash, or just the last modified time could be good enough
    - If we use a file hash, we could include one for the directory to speed up
    finding which subset of the tree contains the change (similar to a Merkle
    Tree).

## Graveyard

### UUID Linking

This was planned to be implemented, and may be in the future, but for now this
plan has been graveyard-ed. It feels like over-engineering at this stage, and we
can ensure that there are no broken links with the `carbon` tool.

To prevent links between markdown notes from breaking when files are moved
around, each markdown notes will have an associated UUIDv4 that will be stored
in YAML front-matter within the note.

```md
---
uuid: be7f834c-56f4-49f1-b19d-4fefeff4561d 
---
# Some note about stuff

blah blah blah
```

These will be automatically added when creating a note with a template, or can
be generated with `carbon uuid new`. 

The IDs will be used instead of paths to notes and will be replaced by the
absolute path to the note at render time. 

To find the path of a note, we will perform a full scan of the `notes-dir/`,
excluding `_rendered/` and `_static/`. This way, UUIDs are explicitly and
directly attached to a file which may move or be renamed (another form of
moving).

We will also include a command `carbon uuid find UUID` where we can provide a
UUID and get back the path of the file that the UUID is attached to.

[boostnote]: https://boostnote.io
[dropbox paper]: https://www.dropbox.com/paper/guide
[notion]: https://www.notion.so
[pulldown-cmark]: https://github.com/raphlinus/pulldown-cmark
[syntect]: https://github.com/trishume/syntect
[mathjax]: https://github.com/mathjax/MathJax
[markdownlint]: https://github.com/DavidAnson/markdownlint
[entr]: https://github.com/clibs/entr
[12-factor-cli-tools]: https://medium.com/@jdxcode/12-factor-cli-apps-dd3c227a0e46
