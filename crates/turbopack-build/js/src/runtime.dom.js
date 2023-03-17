/** @typedef {import('../types/backend').RuntimeBackend} RuntimeBackend */
/** @typedef {import('../types/dom').ChunkResolver} ChunkResolver */
/** @typedef {import('../types').BuildRuntimeParams} BuildRuntimeParams */
/** @typedef {import('../types').ChunkPath} ChunkPath */
/** @typedef {import('../types').SourceInfo} SourceInfo */

/**
 * Maps chunk paths to the corresponding resolver.
 *
 * @type {Map<ChunkPath, ChunkResolver>}
 */
const chunkResolvers = new Map();

/**
 * @param {ChunkPath} chunkPath
 * @returns {ChunkResolver}
 */
function getOrCreateResolver(chunkPath) {
  let resolver = chunkResolvers.get(chunkPath);
  if (!resolver) {
    let resolve;
    let reject;
    const promise = new Promise((innerResolve, innerReject) => {
      resolve = innerResolve;
      reject = innerReject;
    });
    resolver = {
      resolved: false,
      promise,
      resolve: () => {
        resolver.resolved = true;
        resolve();
      },
      reject,
    };
    chunkResolvers.set(chunkPath, resolver);
  }
  return resolver;
}

/** @type {RuntimeBackend} */
let BACKEND;

() => {
  BACKEND = {
    registerChunk(chunkPath, params) {
      registerChunk(chunkPath, params);
    },

    loadChunk(chunkPath, source) {
      return loadChunk(chunkPath, source);
    },
  };

  /**
   * @param {ChunkPath} chunkPath
   * @param {BuildRuntimeParams | undefined} params
   */
  async function registerChunk(chunkPath, params) {
    const resolver = getOrCreateResolver(chunkPath);
    resolver.resolve();

    if (params == null) {
      return;
    }

    if (params.runtimeModuleIds.length > 0) {
      await waitForChunksToLoad(params.otherChunks);

      for (const moduleId of params.runtimeModuleIds) {
        getOrInstantiateRuntimeModule(moduleId, chunkPath);
      }
    }
  }

  /**
   * @param {ChunkPath} chunkPath
   * @param {SourceInfo} source
   * @returns {Promise<void>}
   */
  function loadChunk(chunkPath, source) {
    const resolver = getOrCreateResolver(chunkPath);
    if (resolver.resolved) {
      return resolver.promise;
    }

    // We don't need to load runtime chunks, as they're already
    // present in the DOM.
    if (source.type === SourceTypeRuntime) {
      resolver.resolve();
      return resolver.promise;
    }

    if (chunkPath.endsWith(".css")) {
      const link = document.createElement("link");
      link.rel = "stylesheet";
      link.href = `/${chunkPath}`;
      link.onerror = () => {
        resolver.reject();
      };
      link.onload = () => {
        // CSS chunks do not register themselves, and as such must be marked as
        // loaded instantly.
        resolver.resolve();
      };
      document.body.appendChild(link);
    } else if (chunkPath.endsWith(".js")) {
      const script = document.createElement("script");
      script.src = `/${chunkPath}`;
      // We'll only mark the chunk as resolved once the script has been executed,
      // which happens in `registerChunk`. Hence the absence of `resolve()` in
      // this branch.
      script.onerror = () => {
        resolver.reject();
      };
      document.body.appendChild(script);
    } else {
      throw new Error(`can't infer type of chunk from path ${chunkPath}`);
    }

    return resolver.promise;
  }

  /**
   * @param {ChunkPath[]} chunks
   * @returns {Promise<void[]>}
   */
  function waitForChunksToLoad(chunks) {
    const promises = [];
    for (const chunkPath of chunks) {
      const resolver = getOrCreateResolver(chunkPath);
      if (!resolver.resolved) {
        promises.push(resolver.promise);
      }
    }
    return Promise.all(promises);
  }
};
