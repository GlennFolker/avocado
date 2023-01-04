use crate::incl::*;

pub type AssetLoadSync = Box<dyn Fn(AssetLoadSyncCallback) -> AssetLoadSyncResult>;
pub type AssetLoadSyncCallback = Box<dyn FnOnce(&mut World) -> () + Send + Sync>;
pub type AssetLoadSyncResult = Result<(), anyhow::Error>;

pub trait AssetLoader: 'static + Send + Sync {
    fn load(
        &self,
        reader: Arc<dyn AssetReader>, handle_path: Cow<'static, Path>,
        data: Option<Box<dyn AssetData>>,
        load_sync: AssetLoadSync,
    ) -> Result<Box<dyn AssetDyn>, anyhow::Error>;
}

#[derive(Debug, Error)]
pub enum AssetLoaderError {
    #[error("Asset data is required")]
    NoData,
    #[error("Asset handle exists, but the asset is unloaded")]
    NoAsset,
    #[error("Asset data is in the wrong type")]
    WrongType,
    #[error("{0}")]
    Other(String),
}

pub trait AssetData: 'static + Downcast + Debug + Send + Sync {}
impl_downcast!(AssetData);

impl<T: 'static + Debug + Send + Sync> AssetData for T {}

/// Must only be used with [`std::option::Option::None`].
#[derive(Debug)]
pub struct NoAssetData;
