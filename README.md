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

For more examples, see `tests/`.

## Features

- Fills in Markdown links
- Fast
- Precompiled binaries available
- <150 lines of code

## Non-Features

- No recursive mode, use `find`/`xargs`/shell globs
- Not parallel, use `xargs`, `make`, or `ninja`
