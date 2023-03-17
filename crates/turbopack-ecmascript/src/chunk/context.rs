use anyhow::Result;
use turbo_tasks::{Value, ValueToString};
use turbopack_core::chunk::{
    availability_info::AvailabilityInfo, ChunkItem, ChunkableAssetVc, ChunkingContext,
    ChunkingContextVc, ModuleId, ModuleIdVc,
};

use super::{item::EcmascriptChunkItemVc, EcmascriptChunkPlaceablesVc, EcmascriptChunkRuntimeVc};

/// [`EcmascriptChunkingContext`] must be implemented by [`ChunkingContext`]
/// implementors that want to operate on [`EcmascriptChunk`]s.
#[turbo_tasks::value_trait]
pub trait EcmascriptChunkingContext: ChunkingContext {
    /// Returns an EcmascriptChunkRuntime implementation that registers a
    /// chunk's contents when executed.
    fn ecmascript_runtime(&self) -> EcmascriptChunkRuntimeVc;

    /// Returns an EcmascriptChunkRuntime implementation that registers a
    /// chunk's contents and evaluates its main entries when executed.
    fn evaluated_ecmascript_runtime(
        &self,
        evaluated_entries: EcmascriptChunkPlaceablesVc,
    ) -> EcmascriptChunkRuntimeVc;

    /// Returns the loader item that is used to load the given manifest asset.
    fn manifest_loader_item(
        &self,
        asset: ChunkableAssetVc,
        availability_info: Value<AvailabilityInfo>,
    ) -> EcmascriptChunkItemVc;

    async fn chunk_item_id(&self, chunk_item: EcmascriptChunkItemVc) -> Result<ModuleIdVc> {
        let layer = self.layer();
        let mut ident = chunk_item.asset_ident();
        if !layer.await?.is_empty() {
            ident = ident.with_modifier(layer)
        }
        Ok(ModuleId::String(ident.to_string().await?.clone_value()).cell())
    }
}
