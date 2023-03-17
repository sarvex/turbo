use anyhow::{bail, Result};
use turbo_tasks::{primitives::StringVc, Value, ValueToString, ValueToStringVc};
use turbopack_core::{
    asset::Asset,
    chunk::{ChunkGroupVc, ChunkListReferenceVc, ChunkReferenceVc, ChunkingContext},
    ident::AssetIdentVc,
    reference::AssetReferencesVc,
};
use turbopack_ecmascript::chunk::{
    EcmascriptChunkPlaceableVc, EcmascriptChunkPlaceablesVc, EcmascriptChunkRuntime,
    EcmascriptChunkRuntimeContentVc, EcmascriptChunkRuntimeVc, EcmascriptChunkVc,
    EcmascriptChunkingContextVc,
};

use super::content::EcmascriptBuildNodeChunkContentVc;

/// Development runtime for Ecmascript chunks.
#[turbo_tasks::value(shared)]
pub(crate) struct EcmascriptBuildNodeChunkRuntime {
    /// The chunking context that created this runtime.
    chunking_context: EcmascriptChunkingContextVc,
    /// All chunks of this chunk group need to be ready for execution to start.
    /// When None, it will use a chunk group created from the current chunk.
    chunk_group: Option<ChunkGroupVc>,
    evaluated_entries: Option<EcmascriptChunkPlaceablesVc>,
    exported_entry: Option<EcmascriptChunkPlaceableVc>,
}

#[turbo_tasks::value_impl]
impl EcmascriptBuildNodeChunkRuntimeVc {
    /// Creates a new [`EcmascriptBuildNodeChunkRuntimeVc`].
    #[turbo_tasks::function]
    pub fn new(
        chunking_context: EcmascriptChunkingContextVc,
        evaluated_entries: Option<EcmascriptChunkPlaceablesVc>,
        exported_entry: Option<EcmascriptChunkPlaceableVc>,
    ) -> Self {
        EcmascriptBuildNodeChunkRuntime {
            chunking_context,
            chunk_group: None,
            evaluated_entries,
            exported_entry,
        }
        .cell()
    }
}

#[turbo_tasks::value_impl]
impl ValueToString for EcmascriptBuildNodeChunkRuntime {
    #[turbo_tasks::function]
    async fn to_string(&self) -> Result<StringVc> {
        Ok(StringVc::cell("Ecmascript Build Runtime".to_string()))
    }
}

#[turbo_tasks::function]
fn modifier() -> StringVc {
    StringVc::cell("ecmascript dev chunk".to_string())
}

#[turbo_tasks::value_impl]
impl EcmascriptChunkRuntime for EcmascriptBuildNodeChunkRuntime {
    #[turbo_tasks::function]
    async fn decorate_asset_ident(
        &self,
        origin_chunk: EcmascriptChunkVc,
        ident: AssetIdentVc,
    ) -> Result<AssetIdentVc> {
        let Self {
            chunking_context: _,
            chunk_group,
            evaluated_entries,
            exported_entry,
        } = self;

        let mut ident = ident.await?.clone_value();

        // Add a constant modifier to qualify this runtime.
        ident.add_modifier(modifier());

        // Only add other modifiers when the chunk is evaluated. Otherwise, it will
        // not receive any params and as such won't differ from another chunk in a
        // different chunk group.
        if let Some(evaluated_entries) = evaluated_entries {
            ident.modifiers.extend(
                evaluated_entries
                    .await?
                    .iter()
                    .map(|entry| entry.ident().to_string()),
            );

            // When the chunk group has changed, e.g. due to optimization, we want to
            // include the information too. Since the optimization is
            // deterministic, it's enough to include the entry chunk which is the only
            // factor that influences the chunk group chunks.
            // We want to avoid a cycle when this chunk is the entry chunk.
            if let Some(chunk_group) = chunk_group {
                let entry = chunk_group.entry().resolve().await?;
                if entry != origin_chunk.into() {
                    ident.add_modifier(entry.ident().to_string());
                }
            }
        }

        if let Some(exported_entry) = exported_entry {
            ident.modifiers.push(exported_entry.ident().to_string());
        }

        Ok(AssetIdentVc::new(Value::new(ident)))
    }

    #[turbo_tasks::function]
    fn with_chunk_group(&self, chunk_group: ChunkGroupVc) -> EcmascriptBuildNodeChunkRuntimeVc {
        EcmascriptBuildNodeChunkRuntimeVc::cell(EcmascriptBuildNodeChunkRuntime {
            chunking_context: self.chunking_context,
            chunk_group: Some(chunk_group),
            evaluated_entries: self.evaluated_entries.clone(),
            exported_entry: self.exported_entry.clone(),
        })
    }

    #[turbo_tasks::function]
    async fn references(&self, origin_chunk: EcmascriptChunkVc) -> Result<AssetReferencesVc> {
        Ok(AssetReferencesVc::empty())
        // let Self {
        //     chunk_group,
        //     chunking_context: _,
        //     evaluated_entries,
        //     exported_entry,
        // } = self;

        // let mut references = vec![];
        // if evaluated_entries.is_some() || exported_entry.is_some() {
        //     let chunk_group =
        //         chunk_group.unwrap_or_else(||
        // ChunkGroupVc::from_chunk(origin_chunk.into()));

        //     let chunks = &*chunk_group.chunks().await?;
        //     references.reserve(chunks.len());
        //     for chunk in &*chunk_group.chunks().await? {
        //         if let Some(chunk) =
        // EcmascriptChunkVc::resolve_from(chunk).await? {
        // if chunk == origin_chunk {                 continue;
        //             }
        //         }

        //         references.push(ChunkReferenceVc::new(*chunk).into());
        //     }
        // }
        // Ok(AssetReferencesVc::cell(references))
    }

    #[turbo_tasks::function]
    fn content(&self, origin_chunk: EcmascriptChunkVc) -> EcmascriptChunkRuntimeContentVc {
        EcmascriptBuildNodeChunkContentVc::new(
            origin_chunk,
            self.chunking_context,
            self.chunk_group,
            self.evaluated_entries.clone(),
            self.exported_entry.clone(),
        )
        .into()
    }

    #[turbo_tasks::function]
    async fn merge(
        &self,
        runtimes: Vec<EcmascriptChunkRuntimeVc>,
    ) -> Result<EcmascriptChunkRuntimeVc> {
        let Self {
            chunking_context,
            chunk_group,
            evaluated_entries,
            exported_entry,
        } = self;

        let chunking_context = chunking_context.resolve().await?;
        let chunk_group = if let Some(chunk_group) = chunk_group {
            Some(chunk_group.resolve().await?)
        } else {
            None
        };

        let mut evaluated_entries = if let Some(evaluated_entries) = evaluated_entries {
            Some(evaluated_entries.await?.clone_value())
        } else {
            None
        };

        let mut exported_entry = exported_entry.clone();

        for runtime in runtimes {
            let Some(runtime) = EcmascriptBuildNodeChunkRuntimeVc::resolve_from(runtime).await? else {
                bail!("cannot merge EcmascriptBuildNodeChunkRuntime with non-EcmascriptBuildNodeChunkRuntime");
            };

            let Self {
                chunking_context: other_chunking_context,
                chunk_group: other_chunk_group,
                evaluated_entries: other_evaluated_entries,
                exported_entry: other_exported_entry,
            } = &*runtime.await?;

            let other_chunking_context = other_chunking_context.resolve().await?;
            let other_chunk_group = if let Some(other_chunk_group) = other_chunk_group {
                Some(other_chunk_group.resolve().await?)
            } else {
                None
            };

            if chunking_context != other_chunking_context {
                bail!(
                    "cannot merge EcmascriptBuildNodeChunkRuntime with different chunking contexts",
                );
            }

            if chunk_group != other_chunk_group {
                bail!("cannot merge EcmascriptBuildNodeChunkRuntime with different chunk groups",);
            }

            match (&mut evaluated_entries, other_evaluated_entries) {
                (Some(evaluated_entries), Some(other_evaluated_entries)) => {
                    evaluated_entries.extend(other_evaluated_entries.await?.iter().copied());
                }
                (None, Some(other_evaluated_entries)) => {
                    evaluated_entries = Some(other_evaluated_entries.await?.clone_value());
                }
                _ => {}
            }

            match (&mut exported_entry, other_exported_entry) {
                (Some(exported_entry), Some(other_exported_entry)) => {
                    if exported_entry != other_exported_entry {
                        bail!(
                            "cannot merge EcmascriptBuildNodeChunkRuntime with different exported \
                             entries",
                        );
                    }
                }
                (None, Some(other_exported_entry)) => {
                    exported_entry = Some(*other_exported_entry);
                }
                _ => {}
            }
        }

        Ok(EcmascriptBuildNodeChunkRuntime {
            chunking_context,
            chunk_group,
            evaluated_entries: evaluated_entries.map(EcmascriptChunkPlaceablesVc::cell),
            exported_entry,
        }
        .cell()
        .into())
    }
}
