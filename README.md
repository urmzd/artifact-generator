# artifact-generator

A lightweight local dev tool that watches a file and live-previews it in a browser via Server-Sent Events (SSE). Designed for streaming HTML artifacts token-by-token — useful for benchmarking tokenizers, LLM streaming, and any chunked-write workflow.

## How it works

1. The Rust server watches a file on disk (polling every 100 ms).
2. On every change it broadcasts the full file contents over SSE.
3. The browser shell receives updates and re-renders them inside an `<iframe>` — no page reload needed.
4. When the writer calls `POST /done`, the browser tab closes automatically.

```
writer (Python) ──writes──▶ file ──SSE──▶ browser iframe
                                  ▲
                          artifact-generator (Rust/axum)
```

## Requirements

- [Rust](https://rustup.rs/) (stable)
- [uv](https://github.com/astral-sh/uv) (Python package manager)
- [just](https://github.com/casey/just) (optional, for recipes)

## Quick start

```sh
# Build the server
just build

# Stream a pre-built HTML dashboard (no external deps)
just demo

# Stream via a real LLM (requires ollama)
just demo-llm

# Stream via a HuggingFace tokenizer (gpt2 by default)
just demo-hf

# Stream with BERT tokenizer
just demo-hf tokenizer=bert-base-uncased

# Run offline tokenizer benchmarks (no server needed)
just bench
```

## Recipes

| Recipe | Description |
|---|---|
| `just build` | Compile the Rust server |
| `just demo` | Fixed 30-char chunk streaming, opens browser |
| `just demo-llm` | Live ollama LLM streaming |
| `just demo-hf [tokenizer]` | HuggingFace tokenizer streaming |
| `just bench` | Offline benchmark table: gpt2 vs BERT vs fixed chunks |
| `just test` | Smoke test the server `/content` endpoint |

## HTTP API

| Method | Path | Description |
|---|---|---|
| `GET` | `/` | Browser shell (SSE client) |
| `GET` | `/events` | SSE stream of file contents |
| `GET` | `/content` | Current file contents (plain text) |
| `POST` | `/done` | Signal completion; closes the browser tab |

## Python scripts

All scripts live under `python/` and share zero dependencies except where noted.

| Script | Purpose |
|---|---|
| `python/massive.py` | Generates + streams a large dashboard HTML in 30-char fixed chunks |
| `python/ollama_stream.py` | Streams a live LLM response via ollama |
| `python/benchmarks/hf_stream.py` | Streams via a HuggingFace tokenizer (`tokenizers` package only — no model weights) |
| `python/benchmarks/run.py` | Offline benchmark: tokenize time, token count, streaming throughput |

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
