use std::env;

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS")
        .expect("Target os should be defined");
    match target_os.as_str() {
        "emscripten" => {
            // https://github.com/emscripten-core/emscripten/blob/main/src/settings.js
            println!(
                "cargo:rustc-env=EMCC_CFLAGS=-O3 \
                -s USE_SDL=2 \
                -s FULL_ES3 \
                -s MIN_WEBGL_VERSION=2 \
                -s MAX_WEBGL_VERSION=2 \
                -s STACK_SIZE=10485760"
            );
        }
        _ => {}
    }
}
