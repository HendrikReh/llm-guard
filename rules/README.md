# Rule Packs

Default keyword and regex rule packs live in this directory.

- `keywords.txt`: pipe-delimited records with `id|weight|description|pattern`. Lines beginning with `#` are ignored.
- `patterns.json`: array of objects with `id`, `description`, `pattern`, `weight`, and optional `window`.

These files seed the `FileRuleRepository` implementation and double as examples for creating custom policy packs. Extend them by appending new entries and ensuring `id` values remain unique across both files.
