# bat-treesitter-poc

A reproducible proof-of-concept for [bat#3366 — *Use `tree-sitter` to provide syntax highlighting*](https://github.com/sharkdp/bat/issues/3366).

The maintainers asked for a PoC that could be used for an objective comparison on speed
(including startup), highlighting quality, language coverage, themes, ease of adding custom
languages, binary size and compile time. This repository is that PoC, built on
[`lumis`](https://crates.io/crates/lumis) 0.19.4 (tree-sitter + Neovim themes, native ANSI output).

## TL;DR

**Not recommended as a backend for `bat`.**

The blocker is memory, not CPU. `lumis` renders the whole document in one pass and keeps the
parse tree plus highlight structures alive, so peak memory grows linearly with input size
(roughly 80× the file size). `bat` streams line by line and stays flat at ~9 MB.

| Input | bat 0.26.1 (syntect) | lumis 0.19.4 | Ratio |
|---|---|---|---|
| 100 KB | 8.9 MB | 14.4 MB | 1.6× |
| 1 MB | 8.9 MB | 92.6 MB | 10.4× |
| 10 MB | 9.0 MB | 796.7 MB | **88.5×** |

Switching to the streaming writer (`write_highlight_with_options`) does **not** help:
10 MB peak stayed at 797.1 MB and wall time regressed from 6.6 s to 64.9 s.

## Measurements

Environment: Windows 11, MSVC 14.44.35207, Rust 1.98.1, `bat` v0.26.1 (release build),
`lumis` 0.19.4 (`lang-rust` feature). Times are medians of 5–8 runs; first runs were discarded
because of on-access antivirus scanning (435–711 ms outliers).

### Speed

| Input | bat (syntect) | lumis | Verdict |
|---|---|---|---|
| startup (`--version` / empty file) | 17.2 ms / 26.9 ms | 75.2 ms | lumis 2.8× slower |
| 100 KB | 125 ms | 121 ms | on par |
| 38.9 KB source file (1 035 lines) | 59.5 ms | 72.5 ms | lumis 22% slower |
| 1 MB | 3 072 ms | ~1 300 ms | lumis ~2.4× faster |
| 10 MB | 17 971 ms | 6 601 ms | lumis 2.7× faster |

`lumis` wins on large files but loses on startup and small files, which is what `bat` is
usually used for.

### Highlighting configuration cost

`formatter_build_ms = 0.143` at every input size.

This is worth reporting on its own: [tree-sitter#1942](https://github.com/tree-sitter/tree-sitter/issues/1942)
describes `HighlightConfiguration` construction taking **50–500 ms** per language. `lumis`
does not suffer from it, so that specific concern is resolved in the current ecosystem.

### Binary size and compile time

| | bat 0.26.1 | lumis PoC |
|---|---|---|
| release build | 27m46s | 14m43s (3 languages) |
| binary | 7.68 MB (~200 file extensions) | 5.17 MB (1 language) / 5.57 MB (3) |

`lumis` carries roughly 5 MB of fixed framework cost; each additional grammar is small.
It is not a size win over `bat`, which bundles far more languages.

### Behaviour that matters for a CLI

- `lumis` applies a built-in `Budget` with a default time limit. When exceeded it **silently
  returns the file as plain text** rather than erroring. At 10 MB the output contained
  **zero** ANSI escape sequences. This must be disabled explicitly (`Budget::time_limit(None)`),
  which then exposes the worst case to the user.
- `bat` highlights line by line with a long-lived parser state; `lumis` requires the whole
  document at once. There is no incremental or chunked API exposed.

## Language and theme coverage

| | bat | lumis |
|---|---|---|
| languages | ~200 file extensions (51 `.sublime-syntax` sources + patches) | 121 `lang-*` features |
| themes | ~30 bundled | 120+ (Neovim colorschemes, not `.tmTheme`) |
| adding a language | add one text file | compile a tree-sitter parser and write queries |

Theme count is a genuine win, but the formats are incompatible with `bat`'s existing
`.tmTheme` assets, so it cannot be reused as-is.

## Reproducing

```bash
# 1. build the PoC
cargo build --release

# 2. generate benchmark inputs (100 KB / 1 MB / 10 MB synthesized from bat's sources)
python scripts/gen-bench-files.py

# 3. measure
./target/release/bat-treesitter-poc bench-10mb.rs          # string mode
./target/release/bat-treesitter-poc bench-10mb.rs stream   # streaming writer

# 4. peak memory (Windows)
pwsh -File scripts/measure-peak.ps1 -Exe ./target/release/bat-treesitter-poc.exe -File bench-10mb.rs
```

Compare against a release build of `bat` v0.26.1 with
`bat --style=plain --color=always -p <file>`.

## Limitations

- Only tested on Windows / MSVC / Rust 1.98.1, with `lumis` 0.19.4. Not validated on
  Linux or macOS.
- Highlighting **quality** (scope-by-scope colour comparison) was not evaluated pixel-wise;
  only the presence and count of ANSI escapes was verified.
- `bat`'s 10 MB time in the memory harness is inflated to ~56 s because stdout was redirected
  to a file; the 18 s figure in the speed table is measured with output discarded.
- No claim is made about `syntastica` or other tree-sitter frontends.

## Conclusion

If tree-sitter highlighting is ever revisited for `bat`, it needs:

1. an API that accepts chunks / supports incremental re-parsing, so memory can stay bounded;
2. a way to keep query compilation cheap (already solved by `lumis`);
3. a strategy for large files that does not silently degrade to plain text.

Until then, `syntect` remains the better fit for a streaming CLI.

## License

MIT