use crate::incl::*;

pub trait AssetReader: 'static + Send + Sync {
    fn read_file(&self, path: &Path) -> Result<Vec<u8>, io::Error>;

    fn read_relative(&self, path: &Path, rel: &Path) -> Result<Vec<u8>, io::Error>;
}

#[cfg(feature = "asset_folder")]
pub use folder::AssetFolderReader;
#[cfg(feature = "asset_folder")]
mod folder {
    use super::*;

    pub struct AssetFolderReader {
        asset_folder: Cow<'static, Path>,
    }

    impl AssetFolderReader {
        pub fn new(asset_folder: impl Into<Cow<'static, Path>>) -> Self {
            let root = asset_folder.into();
            match root.try_exists() {
                Ok(true) => if root.is_dir() {
                    Self { asset_folder: root, }
                } else {
                    panic!("Asset folder isn't a directory: {:?}", &root)
                },
                Ok(false) => panic!("Asset folder doesn't exist: {:?}", &root),
                Err(err) => panic!("Invalid asset folder {:?}: {:?}", &root, err),
            }
        }
    }

    impl Default for AssetFolderReader {
        fn default() -> Self {
            match env::var("CARGO_MANIFEST_DIR").ok() {
                Some(v) => {
                    let mut path = Path::new(&v).to_path_buf();
                    path.push("assets");

                    Self::new(path)
                }

                None => Self::new(Path::new("assets"))
            }
        }
    }

    impl AssetReader for AssetFolderReader {
        fn read_file(&self, path: &Path) -> Result<Vec<u8>, io::Error> {
            self.read_relative(path, path)
        }

        fn read_relative(&self, path: &Path, rel: &Path) -> Result<Vec<u8>, io::Error> {
            let buf = self.asset_folder
                .join(path.parent().unwrap_or(Path::new("")))
                .join(rel);

            let mut file = File::open(&buf)?;
            let metadata = file.metadata()?;

            let mut bytes = Vec::with_capacity(metadata.len() as usize);
            file.read_to_end(&mut bytes)?;

            Ok(bytes)
        }
    }
}

#[cfg(feature = "asset_embedded")]
pub use embedded::AssetEmbeddedReader;
#[cfg(feature = "asset_embedded")]
mod embedded {
    use super::*;

    pub struct AssetEmbeddedReader;
    impl AssetReader for AssetEmbeddedReader {
        // TODO
    }
}

cfg_if! {
    if #[cfg(feature = "asset_embedded")] {
        pub type DefaultAssetReader = AssetEmbeddedReader;
    } else if #[cfg(feature = "asset_folder")] {
        pub type DefaultAssetReader = AssetFolderReader;
    } else {
        compile_error!("No asset reader specified.");
    }
}
