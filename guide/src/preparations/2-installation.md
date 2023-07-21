# Installating Tools

### Rust for WebAssembly
Install Rust for `wasm32-unknown-unknown` (the browser WebAssembly target)
```shell
rustup target add wasm32-unknown-unknown
```

### wasm-pack
[wasm-pack](https://rustwasm.github.io/wasm-pack/) is a tool for conveniently
building WASM application.
Install it by one of these options

```shell
# curl the binary
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# or install via cargo (will compile from source)
cargo install wasm-pack

# or use npm/yarn
npm install -g wasm-pack
```

### A Web Server
You need a web server capable of serving the WebAssembly MIME type.
If you don't already have one, you can get
[microserver](https://crates.io/crates/microserver).
```shell
cargo install microserver
```