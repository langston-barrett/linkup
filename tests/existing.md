# Boundaries

Don't modify existing links.

## Inputs

### `existing.md`

```md
[existing](existing.md) links
[should]
not be [modified][modified-link].

[modified-link]: http://foo.bar
[should]: existing.md
```

### Empty files

- `should.md`

## Outputs

### `existing.md`

```md
[existing](existing.md) links
[should]
not be [modified][modified-link].

[modified-link]: http://foo.bar
[should]: existing.md
```

