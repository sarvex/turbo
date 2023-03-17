use anyhow::{bail, Result};
use indexmap::IndexSet;
use indoc::formatdoc;
use turbo_tasks::ValueToString;
use turbopack_core::{
    asset::Asset,
    chunk::{Chunk, ChunkItem, ChunkItemVc, ChunkingContext},
    ident::AssetIdentVc,
    reference::AssetReferencesVc,
};
use turbopack_ecmascript::{
    chunk::{
        EcmascriptChunkItem, EcmascriptChunkItemContent, EcmascriptChunkItemContentVc,
        EcmascriptChunkItemVc, EcmascriptChunkingContextVc,
    },
    utils::StringifyJs,
};

use super::chunk_asset::BuildManifestChunkAssetVc;

/// The BuildManifestChunkItem generates a __turbopack_load__ call for every
/// chunk necessary to load the real asset. Once all the loads resolve, it is
/// safe to __turbopack_import__ the actual module that was dynamically
/// imported.
#[turbo_tasks::value(shared)]
pub(super) struct BuildManifestChunkItem {
    pub context: EcmascriptChunkingContextVc,
    pub manifest: BuildManifestChunkAssetVc,
}

#[turbo_tasks::value_impl]
impl EcmascriptChunkItem for BuildManifestChunkItem {
    #[turbo_tasks::function]
    fn chunking_context(&self) -> EcmascriptChunkingContextVc {
        self.context
    }

    #[turbo_tasks::function]
    async fn content(&self) -> Result<EcmascriptChunkItemContentVc> {
        let chunk_group = self.manifest.chunk_group();
        let chunks = chunk_group.chunks().await?;
        let output_root = self.context.output_root().await?;

        let mut chunk_server_paths = IndexSet::new();
        for chunk in chunks.iter() {
            // The "path" in this case is the chunk's path, not the chunk item's path.
            // The difference is a chunk is a file served by the dev server, and an
            // item is one of several that are contained in that chunk file.
            let chunk_path = &*chunk.path().await?;
            // The pathname is the file path necessary to load the chunk from the server.
            let chunk_server_path = if let Some(path) = output_root.get_path_to(chunk_path) {
                path
            } else {
                bail!(
                    "chunk path {} is not in output root {}",
                    chunk.path().to_string().await?,
                    self.context.output_root().to_string().await?
                );
            };
            chunk_server_paths.insert(chunk_server_path.to_string());
        }

        let code = formatdoc! {
            r#"
                __turbopack_export_value__({:#});
            "#,
            StringifyJs(&chunk_server_paths)
        };

        Ok(EcmascriptChunkItemContent {
            inner_code: code.into(),
            ..Default::default()
        }
        .into())
    }
}

#[turbo_tasks::value_impl]
impl ChunkItem for BuildManifestChunkItem {
    #[turbo_tasks::function]
    fn asset_ident(&self) -> AssetIdentVc {
        self.manifest.ident()
    }

    #[turbo_tasks::function]
    fn references(&self) -> AssetReferencesVc {
        self.manifest.references()
    }
}
