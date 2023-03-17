import {
  GetFirstModuleChunk,
  InstantiateRuntimeModule,
  SourceType,
} from "types";

export { RuntimeBackend } from "types";

declare global {
  declare const RUNTIME_CHUNK_PATH: ChunkPath;
  declare const getFirstModuleChunk: GetFirstModuleChunk;
  declare const instantiateRuntimeModule: InstantiateRuntimeModule;
  declare const getOrInstantiateRuntimeModule: InstantiateRuntimeModule;
  declare const SourceTypeRuntime: SourceType.Runtime;
  declare const SourceTypeParent: SourceType.Parent;
}
