# tree-sitter-rpu

Tree-sitter grammar for the RPU scene and scripting language.

## Scope

This grammar is intended to be the single syntax source of truth for:

- Zed editor support
- website syntax highlighting
- future parser-driven tooling such as outline views, diagnostics helpers, or formatting

It covers the current mixed RPU language surface:

- scene files with `scene`, `camera`, `rect`, `sprite`, `text`, `map`
- embedded script blocks inside visual nodes
- standalone scripts with `state`, `fn`, `on`, `if`, `else`, `let`, assignments, calls, and expressions

## Development

```bash
cd tree-sitter-rpu
npm install
npm run generate
npm test
```

## Notes

This is intentionally scoped to the current language, not to every possible future syntax idea.
When RPU syntax changes, update this grammar first and reuse it across integrations rather than maintaining separate highlighters.
