# Boundaries

Test linking at the boundaries.

## Inputs

### `first.md`

```md
[linking] should work at the beginning of files
```

### `last.md`

```md
linking should work at the beginning of [files]
```

### `whole.md`

```md
[linking should work for a whole file]
```

### Empty files

- `linking.md`
- `files.md`
- `linking-should-work-for-a-whole-file.md`

## Outputs

### `first.md`

```md
[linking](linking.md) should work at the beginning of files
```

### `last.md`

```md
linking should work at the beginning of [files](files.md)
```

### `whole.md`

```md
[linking should work for a whole file](linking-should-work-for-a-whole-file.md)
```
