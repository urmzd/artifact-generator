#!/usr/bin/env python3
"""
Streams a large pre-built HTML dashboard via a HuggingFace tokenizer, token by token.

Usage: uv run --project python python/benchmarks/hf_stream.py [output-path] [port] [tokenizer]
"""
import sys, time, urllib.request
from tokenizers import Tokenizer

# Add parent directory to path so we can import massive.py
sys.path.insert(0, __file__.rsplit("/", 2)[0])
from massive import build_html

PATH      = sys.argv[1] if len(sys.argv) > 1 else "/tmp/artifact.html"
PORT      = sys.argv[2] if len(sys.argv) > 2 else None
TOK_NAME  = sys.argv[3] if len(sys.argv) > 3 else "gpt2"

print(f"Tokenizer : {TOK_NAME}")
print(f"Output    : {PATH}")
print(f"Loading tokenizer...", end=" ", flush=True)
tok = Tokenizer.from_pretrained(TOK_NAME)
print("done")

html     = build_html()
encoding = tok.encode(html)
tokens   = [tok.decode([id]) for id in encoding.ids]

total_tokens = len(tokens)
total_bytes  = len(html.encode())
avg_chars    = len(html) / total_tokens if total_tokens else 0

print(f"Corpus    : {total_bytes:,} bytes  |  {total_tokens:,} tokens  |  avg {avg_chars:.1f} chars/token")
print(f"Streaming...", end=" ", flush=True)

flushes = 0
t0      = time.perf_counter()

with open(PATH, "w") as f:
    for token in tokens:
        f.write(token)
        f.flush()
        flushes += 1
        if flushes % 1000 == 0:
            print(".", end="", flush=True)

elapsed  = time.perf_counter() - t0
kb       = total_bytes / 1024
kbps     = kb / elapsed if elapsed > 0 else 0
toks_sec = total_tokens / elapsed if elapsed > 0 else 0
sse_est  = max(1, int(elapsed / 0.1))

print(f"\n\n{'─'*44}")
print(f"  Tokenizer     : {TOK_NAME}")
print(f"  Bytes written : {total_bytes:>10,}")
print(f"  Tokens        : {total_tokens:>10,}")
print(f"  Avg chars/tok : {avg_chars:>10.1f}")
print(f"  Elapsed       : {elapsed:>10.2f} s")
print(f"  Throughput    : {kbps:>10.1f} KB/s")
print(f"  Tokens/sec    : {toks_sec:>10.0f}")
print(f"  Flushes       : {flushes:>10,}")
print(f"  SSE updates ~ : {sse_est:>10,}")
print(f"{'─'*44}")

if PORT:
    urllib.request.urlopen(f"http://localhost:{PORT}/done", data=b"")
