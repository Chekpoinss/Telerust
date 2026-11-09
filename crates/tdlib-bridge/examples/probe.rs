use std::path::PathBuf;

fn main() {
    let Some(path) = std::env::args_os().nth(1) else {
        eprintln!(
            "usage: cargo run -p telerust-tdlib-bridge --example probe -- /path/to/libtdjson.so"
        );
        std::process::exit(2);
    };
    match telerust_tdlib_bridge::probe_tdlib_library(&PathBuf::from(path)) {
        Ok(()) => {
            println!("TDLib JSON C ABI loaded and responded; no authorization or network used.")
        }
        Err(error) => {
            eprintln!("TDLib probe failed: {error}");
            std::process::exit(1);
        }
    }
}
