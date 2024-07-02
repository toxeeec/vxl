use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    prelude::*,
    utils::BoxedFuture,
};
use std::str;
use toml::{map::Map, Table, Value};

#[derive(Asset, TypePath, Debug)]
pub(super) struct TomlAsset(pub(super) Map<String, Value>);

#[derive(Default, Debug)]
pub(super) struct TomlLoader;

impl AssetLoader for TomlLoader {
    type Asset = TomlAsset;
    type Settings = ();
    type Error = anyhow::Error;
    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a (),
        _load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<TomlAsset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let str = String::from_utf8(bytes)?;
            let table = str.parse::<Table>()?;
            Ok(TomlAsset(table))
        })
    }

    fn extensions(&self) -> &[&str] {
        &["toml"]
    }
}
