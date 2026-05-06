# Semantic Benchmarks

## Snake benchmarks

Current benchmark examples:

- `snake_core.sm` - deterministic headless snake engine.
- `snake_learning.sm` - deterministic Q-learning training loop.

## Trace adapter contract

Trace adapter contract:

- `../../docs/roadmap/snake_trace_adapter_contract.md`

Sample trace:

- `snake_trace_sample.txt`

## Boundary

Semantic emits deterministic text traces.

External renderers consume those traces.

Browser, DOM, Canvas, WebGL, native windows, animation timing, and UI lifecycle are outside the Semantic language boundary.
