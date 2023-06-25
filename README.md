# SIMG
A simple graphics library for fast prototyping.

## Features
- 2D and 3D batch rendering
- Textures
- Fonts rendering and ttf/otf glyph atlases construction
- WebAssembly build

## Examples
### Native
To build and run an example execute (check possible example names in the `./examples` dir):
```bash
cargo run --release --example breakout
```
Also, you can use python build script for this:
```bash
python tools/build.py -e example_name -r
```

### WebAssembly
Make sure that you have emscripten compiler [installed](https://www.hellorust.com/setup/emscripten/).

To build an example for WebAssembly execute:
```bash
cargo build --release --target wasm32-unknown-emscripten --example breakout
```
Or with python script:
```bash
python tools/build.py -e example_name -w
```

To run it in browser modify the `./examples/wasm/index.html` such that it uses `.js` script for your example:
```html
<script src="src/example_name.js"></script>
```

Run any http server in `./examples/wasm` directory:
```bash
cd ./examples/wasm && python -m http.server
```
