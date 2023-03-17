use anyhow::Result;
use turbo_tasks::{primitives::StringVc, Value};
use turbopack_core::{
    asset::{Asset, AssetContentVc, AssetVc},
    chunk::{
        availability_info::AvailabilityInfo, ChunkGroupVc, ChunkReferenceVc, ChunkVc,
        ChunkableAsset, ChunkableAssetVc, ChunkingContextVc,
    },
    ident::AssetIdentVc,
    reference::AssetReferencesVc,
};
use turbopack_ecmascript::chunk::{
    EcmascriptChunkItemVc, EcmascriptChunkPlaceable, EcmascriptChunkPlaceableVc, EcmascriptChunkVc,
    EcmascriptChunkingContextVc, EcmascriptExports, EcmascriptExportsVc,
};

use super::chunk_item::BuildManifestChunkItem;

#[turbo_tasks::function]
fn modifier() -> StringVc {
    StringVc::cell("manifest chunk".to_string())
}

/// The manifest chunk is deferred until requested by the manifest loader
/// item when the dynamic `import()` expression is reached. Its responsibility
/// is to generate a Promise that will resolve only after all the necessary
/// chunks needed by the dynamic import are loaded by the client.
///
/// Splitting the dynamic import into a quickly generate-able manifest loader
/// item and a slow-to-generate manifest chunk allows for faster incremental
/// compilation. The traversal won't be performed until the dynamic import is
/// actually reached, instead of eagerly as part of the chunk that the dynamic
/// import appears in.
#[turbo_tasks::value(shared)]
pub struct BuildManifestChunkAsset {
    pub asset: ChunkableAssetVc,
    pub chunking_context: EcmascriptChunkingContextVc,
    pub availability_info: AvailabilityInfo,
}

#[turbo_tasks::value_impl]
impl BuildManifestChunkAssetVc {
    #[turbo_tasks::function]
    pub fn new(
        asset: ChunkableAssetVc,
        chunking_context: EcmascriptChunkingContextVc,
        availability_info: Value<AvailabilityInfo>,
    ) -> Self {
        Self::cell(BuildManifestChunkAsset {
            asset,
            chunking_context,
            availability_info: availability_info.into_value(),
        })
    }

    #[turbo_tasks::function]
    pub(super) async fn chunk_group(self) -> Result<ChunkGroupVc> {
        let this = self.await?;
        Ok(ChunkGroupVc::from_asset(
            this.asset,
            this.chunking_context.into(),
            Value::new(this.availability_info),
        ))
    }

    #[turbo_tasks::function]
    pub async fn manifest_chunk(self) -> Result<ChunkVc> {
        let this = self.await?;
        Ok(self.as_chunk(
            this.chunking_context.into(),
            Value::new(this.availability_info),
        ))
    }
}

#[turbo_tasks::value_impl]
impl Asset for BuildManifestChunkAsset {
    #[turbo_tasks::function]
    fn ident(&self) -> AssetIdentVc {
        self.asset.ident().with_modifier(modifier())
    }

    #[turbo_tasks::function]
    fn content(&self) -> AssetContentVc {
        todo!()
    }

    #[turbo_tasks::function]
    async fn references(self_vc: BuildManifestChunkAssetVc) -> Result<AssetReferencesVc> {
        let chunk_group = self_vc.chunk_group();
        let chunks = chunk_group.chunks();

        Ok(AssetReferencesVc::cell(
            chunks
                .await?
                .iter()
                .copied()
                .map(ChunkReferenceVc::new)
                .map(Into::into)
                .collect(),
        ))
    }
}

#[turbo_tasks::value_impl]
impl ChunkableAsset for BuildManifestChunkAsset {
    #[turbo_tasks::function]
    fn as_chunk(
        self_vc: BuildManifestChunkAssetVc,
        context: ChunkingContextVc,
        availability_info: Value<AvailabilityInfo>,
    ) -> ChunkVc {
        EcmascriptChunkVc::new(context, self_vc.into(), availability_info).into()
    }
}

#[turbo_tasks::value_impl]
impl EcmascriptChunkPlaceable for BuildManifestChunkAsset {
    #[turbo_tasks::function]
    fn as_chunk_item(
        self_vc: BuildManifestChunkAssetVc,
        context: EcmascriptChunkingContextVc,
    ) -> EcmascriptChunkItemVc {
        BuildManifestChunkItem {
            context,
            manifest: self_vc,
        }
        .cell()
        .into()
    }

    #[turbo_tasks::function]
    fn get_exports(&self) -> EcmascriptExportsVc {
        EcmascriptExports::Value.cell()
    }
}
