import {
  createOnMessage as __wasmCreateOnMessageForFsProxy,
  getDefaultContext as __emnapiGetDefaultContext,
  instantiateNapiModuleSync as __emnapiInstantiateNapiModuleSync,
  WASI as __WASI,
} from '@napi-rs/wasm-runtime'



const __wasi = new __WASI({
  version: 'preview1',
})

const __wasmUrl = new URL('./hyperjs.wasm32-wasi.wasm', import.meta.url).href
const __emnapiContext = __emnapiGetDefaultContext()


const __sharedMemory = new WebAssembly.Memory({
  initial: 4000,
  maximum: 65536,
  shared: true,
})

const __wasmFile = await fetch(__wasmUrl).then((res) => res.arrayBuffer())

const {
  instance: __napiInstance,
  module: __wasiModule,
  napiModule: __napiModule,
} = __emnapiInstantiateNapiModuleSync(__wasmFile, {
  context: __emnapiContext,
  asyncWorkPoolSize: 4,
  wasi: __wasi,
  onCreateWorker() {
    const worker = new Worker(new URL('./wasi-worker-browser.mjs', import.meta.url), {
      type: 'module',
    })

    return worker
  },
  overwriteImports(importObject) {
    importObject.env = {
      ...importObject.env,
      ...importObject.napi,
      ...importObject.emnapi,
      memory: __sharedMemory,
    }
    return importObject
  },
  beforeInit({ instance }) {
    for (const name of Object.keys(instance.exports)) {
      if (name.startsWith('__napi_register__')) {
        instance.exports[name]()
      }
    }
  },
})
export default __napiModule.exports
export const Body = __napiModule.exports.Body
export const Bytes = __napiModule.exports.Bytes
export const Full = __napiModule.exports.Full
export const Method = __napiModule.exports.Method
export const Request = __napiModule.exports.Request
export const RequestBuilder = __napiModule.exports.RequestBuilder
export const Builder = __napiModule.exports.Builder
export const RequestContext = __napiModule.exports.RequestContext
export const Response = __napiModule.exports.Response
export const ResponseBuilder = __napiModule.exports.ResponseBuilder
export const Builder = __napiModule.exports.Builder
export const Router = __napiModule.exports.Router
export const Server = __napiModule.exports.Server
export const StatusCode = __napiModule.exports.StatusCode
export const Version = __napiModule.exports.Version
export const completeRequest = __napiModule.exports.completeRequest
