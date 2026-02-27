# Contributing

Contributions are welcome. Please follow these guidelines.

## Getting started

```sh
git clone <repo-url>
cd artifact-generator
just build   # compile the Rust server
just test    # run the smoke test
just bench   # run offline tokenizer benchmarks
```

## Project structure

```
artifact-generator/
├── src/main.rs                   # Rust/axum SSE server
├── python/
│   ├── massive.py                # fixed-chunk streaming demo + build_html()
│   ├── ollama_stream.py          # live LLM streaming via ollama
│   └── benchmarks/
│       ├── hf_stream.py          # HuggingFace tokenizer streaming
│       └── run.py                # offline benchmark comparison
├── justfile                      # task recipes
└── python/pyproject.toml         # Python dependencies (uv)
```

## Making changes

- **Rust server** (`src/main.rs`): keep it dependency-light; axum + tokio only.
- **Python scripts**: import `build_html()` from `massive.py` rather than duplicating the corpus.
- **New tokenizers**: add them to the `TOKENIZERS` list in `python/benchmarks/run.py`.
- **New recipes**: add them to `justfile` with a comment describing what they do.

## Pull requests

1. Fork the repo and create a branch from `main`.
2. Make your changes and ensure `just test` passes.
3. For Python changes, run `just bench` to confirm benchmarks still execute.
4. Open a pull request with a clear description of what changed and why.

## Code style

- Rust: `cargo fmt` before committing.
- Python: keep scripts self-contained and runnable via `uv run --project python`.

## License

By contributing you agree that your contributions will be licensed under the [Apache 2.0 License](LICENSE).
