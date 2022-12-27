use cargo_emit::*;

use std::{
    env::var,
    path::Path,
};

const ASSET_VAR: &str = "AVOCADO_ASSET_FOLDER";

#[allow(unreachable_code)]
fn main() {
    #[cfg(not(feature = "asset_embedded"))]
    return;

    rerun_if_env_changed!(ASSET_VAR);
    let dir = var(ASSET_VAR).ok()
        .map(|v| Path::new(&v).to_path_buf())
        .and_then(|path| {
            if path.exists() && match path.file_name() {
                Some(name) => name == "assets",
                None => false,
            } {
                Some(path)
            } else {
                warning!("Asset folder pointed by {} doesn't exist or invalid: {:?}", ASSET_VAR, &path);
                None
            }
        })
        .or_else(|| {
            let dir = var("CARGO_MANIFEST_DIR").expect("No manifest directory supplied. Did you properly compile the project?");
            let mut dir = Path::new(&dir).to_path_buf();
            dir.push("assets");

            if dir.exists() {
                Some(dir)
            } else {
                panic!("\
                    Asset folder not found; make sure you have a folder named `assets` in your project root directory \
                    or configure it with the `{}` environment variable.
                ", ASSET_VAR);
            }
        })
        .unwrap();

    rerun_if_changed!(dir.to_string_lossy());
    warning!("Asset folder: {:?}", &dir);

    // TODO
}
