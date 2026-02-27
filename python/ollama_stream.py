#!/usr/bin/env python3
"""
Streams a large self-contained HTML dashboard from an ollama model to the
watched file, token by token.

Usage: uv run python/ollama_stream.py [output-path] [model]
"""
import sys, time, urllib.request, ollama

PATH  = sys.argv[1] if len(sys.argv) > 1 else "/tmp/artifact.html"
MODEL = sys.argv[2] if len(sys.argv) > 2 else "gemma3"
PORT  = sys.argv[3] if len(sys.argv) > 3 else None

PROMPT = """Create a large, self-contained HTML dashboard page with inline CSS only (no external resources).

Include:
- Top navigation bar with a logo, nav links, and a signed-in user avatar with a Sign Out button
- Left sidebar with grouped navigation links (Main, Management, System sections)
- Dashboard overview with 4 stat cards (Total Users, Revenue, Orders, Uptime)
- Users table with at least 50 rows of realistic fake data (name, email, role, status badge, joined date)
- Orders table with at least 40 rows (order ID, product, amount, date, status badge with color)
- Account settings form with fields (name, email, phone, role, timezone, bio, password), notification toggles, and a Danger Zone delete section

Requirements:
- All CSS must be inline in a <style> tag — no CDN, no external fonts, no imports
- Use a clean, modern design with a light grey background
- Status badges should use colored pill spans
- The page must be fully self-contained and render correctly in an iframe without network access

Output raw HTML only. No markdown fences, no explanation."""

print(f"Model : {MODEL}")
print(f"Output: {PATH}")
print("Streaming", end="", flush=True)

bytes_written = 0
flushes       = 0
t0            = time.perf_counter()

with open(PATH, "w") as f:
    for chunk in ollama.generate(model=MODEL, prompt=PROMPT, stream=True):
        token = chunk.get("response", "")
        if token:
            f.write(token)
            f.flush()
            bytes_written += len(token.encode())
            flushes += 1
            if flushes % 100 == 0:
                print(".", end="", flush=True)

elapsed = time.perf_counter() - t0
kb      = bytes_written / 1024
kbps    = kb / elapsed if elapsed > 0 else 0
sse_est = max(1, int(elapsed / 0.1))

print(f"\n\n{'─'*44}")
print(f"  Bytes written : {bytes_written:>10,}")
print(f"  Elapsed       : {elapsed:>10.2f} s")
print(f"  Throughput    : {kbps:>10.1f} KB/s")
print(f"  Flushes       : {flushes:>10,}")
print(f"  SSE updates ~ : {sse_est:>10,}")
print(f"{'─'*44}")

if PORT:
    urllib.request.urlopen(f"http://localhost:{PORT}/done", data=b"")
