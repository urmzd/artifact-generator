build:
    cargo build

install:
    cargo install --path .

run file="/tmp/artifact.html" port="3000":
    cargo run -- {{file}} --port {{port}}

# Sanity test: stream pre-built HTML (no external deps)
demo file="/tmp/artifact.html" port="3000": build
    lsof -ti :{{port}} | xargs kill -9 2>/dev/null || true
    ./target/debug/artifact-generator {{file}} --port {{port}} &
    sleep 0.3
    open http://localhost:{{port}}
    uv run --project python python/massive.py {{file}} {{port}}

# Real LLM stream via ollama
demo-llm file="/tmp/artifact.html" port="3000" model="gemma3": build
    lsof -ti :{{port}} | xargs kill -9 2>/dev/null || true
    ./target/debug/artifact-generator {{file}} --port {{port}} &
    sleep 0.3
    open http://localhost:{{port}}
    uv run --project python python/ollama_stream.py {{file}} {{model}} {{port}}

# Offline tokenizer benchmarks — no server needed
bench:
    uv run --project python python/benchmarks/run.py

# Live preview with HF tokenizer streaming
demo-hf tokenizer="gpt2" file="/tmp/artifact.html" port="3000": build
    lsof -ti :{{port}} | xargs kill -9 2>/dev/null || true
    ./target/debug/artifact-generator {{file}} --port {{port}} &
    sleep 0.3
    open http://localhost:{{port}}
    uv run --project python python/benchmarks/hf_stream.py {{file}} {{port}} {{tokenizer}}

test: build
    #!/usr/bin/env bash
    set -e
    TEST_FILE=$(mktemp /tmp/artifact-test-XXXX.html)
    PORT=13000
    echo "<h1>just test</h1>" > "$TEST_FILE"
    lsof -ti :$PORT | xargs kill -9 2>/dev/null || true
    ./target/debug/artifact-generator "$TEST_FILE" --port $PORT &
    SERVER_PID=$!
    sleep 0.5
    RESULT=$(curl -sf "http://localhost:$PORT/content")
    kill "$SERVER_PID" 2>/dev/null
    rm -f "$TEST_FILE"
    if echo "$RESULT" | grep -q "just test"; then
        echo "PASS: content endpoint returned expected output"
    else
        echo "FAIL: unexpected output: $RESULT"
        exit 1
    fi
