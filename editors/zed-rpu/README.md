# RPU Zed Extension

This is a local Zed development extension for `.rpu` files. It uses a local Git mirror of the repository's `tree-sitter-rpu` grammar through Zed's `file://` grammar support.

## Install In Zed

1. Open the command palette.
2. Run `zed: install dev extension`.
3. Select `/Users/markusmoenig/RPU/editors/zed-rpu`.
4. Open any `.rpu` file and select `RPU` if Zed does not pick it automatically.

## Update Grammar

After changing `tree-sitter-rpu/grammar.js`, regenerate and test the parser:

```sh
cd /Users/markusmoenig/RPU/tree-sitter-rpu
npm run generate
npm test
./node_modules/.bin/tree-sitter parse --quiet ../examples/sunnyland/scenes/main.rpu ../examples/terrain/scenes/main.rpu
```

Then reload the dev extension or restart Zed if the parser cache does not refresh.

The grammar path in `extension.toml` is absolute for this local checkout. If the repository moves, update `repository = "file:///..."`.

Zed checks out the grammar at the configured `rev`, so `rev` must be a real commit hash. For local uncommitted grammar work, refresh the ignored local grammar mirror:

```sh
cd /Users/markusmoenig/RPU
rm -rf editors/zed-rpu/local-tree-sitter-rpu editors/zed-rpu/grammars
mkdir -p editors/zed-rpu/local-tree-sitter-rpu
rsync -a --exclude node_modules --exclude .git tree-sitter-rpu/ editors/zed-rpu/local-tree-sitter-rpu/
git -C editors/zed-rpu/local-tree-sitter-rpu init
git -C editors/zed-rpu/local-tree-sitter-rpu config user.name "RPU Zed Dev"
git -C editors/zed-rpu/local-tree-sitter-rpu config user.email "zed-dev@local"
git -C editors/zed-rpu/local-tree-sitter-rpu add .
git -C editors/zed-rpu/local-tree-sitter-rpu commit -m "Local RPU tree-sitter grammar"
git -C editors/zed-rpu/local-tree-sitter-rpu rev-parse HEAD
```

Then update `rev` in `extension.toml` to the printed commit hash and reinstall/reload the dev extension.
