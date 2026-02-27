# artifact-generator

A local dev tool that watches an HTML file on disk and continuously renders it to PDF using headless Chrome. Designed for streaming HTML artifacts token-by-token — useful for benchmarking tokenizers, LLM streaming, and any chunked-write workflow.

## How it works

1. The Rust binary watches a file on disk (polling every 100 ms).
2. On every change it renders the HTML to PDF via headless Chrome.
3. The PDF is overwritten in place on each render cycle.

```
writer (Python) ──writes──▶ file.html ──render──▶ file.pdf
                                        ▲
                              artifact-generator (Rust + headless Chrome)
```

## Requirements

- [Rust](https://rustup.rs/) (stable)
- [Google Chrome](https://www.google.com/chrome/) or Chromium (headless rendering)
- [uv](https://github.com/astral-sh/uv) (Python package manager)
- [just](https://github.com/casey/just) (optional, for recipes)

## Quick start

```sh
# Build the binary
just build

# Stream a pre-built HTML dashboard and produce a PDF
just demo

# Stream via a real LLM (requires ollama)
just demo-llm

# Stream via a HuggingFace tokenizer (gpt2 by default)
just demo-hf

# Stream with BERT tokenizer
just demo-hf tokenizer=bert-base-uncased

# Run offline tokenizer benchmarks (no server needed)
just bench

# Run Rust criterion benchmarks (file watcher, broadcast throughput)
just bench-rust
```

## CLI usage

```sh
artifact-generator <input.html> [--output output.pdf]
```

- `<input.html>` — the HTML file to watch.
- `--output` — optional PDF output path (defaults to `<input>.pdf`).

The process runs until interrupted with Ctrl+C.

## Recipes

| Recipe | Description |
|---|---|
| `just build` | Compile the Rust binary |
| `just install` | Install the binary via `cargo install` |
| `just demo` | Stream a pre-built HTML dashboard, produce PDF |
| `just demo-llm [model]` | Live ollama LLM streaming (default: gemma3) |
| `just demo-hf [tokenizer]` | HuggingFace tokenizer streaming |
| `just bench` | Offline Python tokenizer benchmarks |
| `just bench-rust` | Rust criterion benchmarks (watcher, broadcast) |
| `just test` | Smoke test: verify PDF output is produced |

## Python package

The Python scripts live under `python/` and are structured as a proper package (`artifact_generator`) with console entry points:

| Entry point | Description |
|---|---|
| `ag-demo` | Stream a pre-built HTML dashboard in fixed chunks |
| `ag-ollama` | Stream a live LLM response via ollama |
| `ag-stream` | Generic file streaming utility |
| `ag-hf-stream` | Stream via a HuggingFace tokenizer |
| `ag-bench` | Offline benchmark: tokenize time, token count, throughput |
| `ag-realtime` | Real-time streaming dashboard |

Install and run any entry point with:

```sh
uv run --project python ag-bench
```

## Benchmark output (example)

```
Tokenizer                    Tokens  Avg ch/tok    Tok ms   Tokens/sec
────────────────────────────────────────────────────────────────────────
gpt2                         27,300         2.4      14.2       1,917k
bert-base-uncased            33,628         1.9      16.1       2,094k
Fixed 30-char chunks          2,169        30.0       0.1      23,220k
```

## License

Apache 2.0 — see [LICENSE](LICENSE).
