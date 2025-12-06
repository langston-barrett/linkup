# linkup

`linkup` automatically adds local links to Markdown files. Given Markdown files
like so:
```md
<!-- lorem.md -->
Lorem [ipsum] dolor sit amet.
```
```md
<!-- ipsum.md -->
Consectetur adipiscing elit.
```
It will fill in the link to `ipsum`:
```md
<!-- lorem.md -->
Lorem [ipsum](ipsum.md) dolor sit amet.
```

## Features

- Fills in Markdown links
- Fast
- Precompiled binaries available
- Safe, doesn't panic
- <XXX lines of code

## Non-Features

- No recursive mode, use `find`/`xargs`/shell globs
- Not parallel, use `xargs`, `make`, or `ninja`

## Install

Download a binary from the [releases page][releases], or build with
[Cargo][cargo]:

```sh
cargo install --locked linkup
```

[cargo]: https://doc.rust-lang.org/cargo/
[releases]: https://github.com/langston-barrett/linkup/releases
