use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    prelude::*,
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
    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _settings: &'a Self::Settings,
        _load_context: &'a mut LoadContext<'_>,
    ) -> Result<TomlAsset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let str = String::from_utf8(bytes)?;
        let table = str.parse::<Table>()?;
        Ok(TomlAsset(table))
    }

    fn extensions(&self) -> &[&str] {
        &["toml"]
    }
}
