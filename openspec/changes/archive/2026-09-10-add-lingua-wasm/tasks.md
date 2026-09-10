# Tasks — add-lingua-wasm

## 1. WASM target (spec lingua-analysis — native/WASM parity)

- [x] 1.1 `lingua-wasm` crate/feature: wasm-bindgen bindings (batch-of-blocks analysis → classified tokens, statuses, percentage, glosses); `wasm-pack --target web` build
- [x] 1.2 Native/WASM parity test over the fixtures (same `analyzer_version` ⇒ identical output)
- [x] 1.3 CI lane: build the wasm module + run the parity tests
