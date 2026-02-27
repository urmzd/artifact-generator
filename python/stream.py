#!/usr/bin/env python3
"""
Usage: uv run python/stream.py [output-path] [model]
"""
import sys, ollama

PATH  = sys.argv[1] if len(sys.argv) > 1 else "/tmp/artifact.html"
MODEL = sys.argv[2] if len(sys.argv) > 2 else "llama3.2"

PROMPT = """Create a self-contained HTML page with CSS animations.
Output raw HTML only, no markdown fences."""

with open(PATH, "w") as f:
    for chunk in ollama.generate(model=MODEL, prompt=PROMPT, stream=True):
        token = chunk.get("response", "")
        if token:
            f.write(token)
            f.flush()  # flush each token so watcher sees changes
