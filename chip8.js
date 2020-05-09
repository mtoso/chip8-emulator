
let wasm;

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });

cachedTextDecoder.decode();

let cachegetUint8Memory0 = null;
function getUint8Memory0() {
    if (cachegetUint8Memory0 === null || cachegetUint8Memory0.buffer !== wasm.memory.buffer) {
        cachegetUint8Memory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachegetUint8Memory0;
}

function getStringFromWasm0(ptr, len) {
    return cachedTextDecoder.decode(getUint8Memory0().subarray(ptr, ptr + len));
}

let WASM_VECTOR_LEN = 0;

function passArray8ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 1);
    getUint8Memory0().set(arg, ptr / 1);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

let cachegetInt32Memory0 = null;
function getInt32Memory0() {
    if (cachegetInt32Memory0 === null || cachegetInt32Memory0.buffer !== wasm.memory.buffer) {
        cachegetInt32Memory0 = new Int32Array(wasm.memory.buffer);
    }
    return cachegetInt32Memory0;
}

function getArrayU8FromWasm0(ptr, len) {
    return getUint8Memory0().subarray(ptr / 1, ptr / 1 + len);
}

function _assertClass(instance, klass) {
    if (!(instance instanceof klass)) {
        throw new Error(`expected instance of ${klass.name}`);
    }
    return instance.ptr;
}
/**
*/
export class Cartridge {

    static __wrap(ptr) {
        const obj = Object.create(Cartridge.prototype);
        obj.ptr = ptr;

        return obj;
    }

    free() {
        const ptr = this.ptr;
        this.ptr = 0;

        wasm.__wbg_cartridge_free(ptr);
    }
    /**
    * @param {Uint8Array} data
    * @returns {Cartridge}
    */
    static new(data) {
        var ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_malloc);
        var len0 = WASM_VECTOR_LEN;
        var ret = wasm.cartridge_new(ptr0, len0);
        return Cartridge.__wrap(ret);
    }
    /**
    * @returns {Uint8Array}
    */
    get_memory() {
        wasm.cartridge_get_memory(8, this.ptr);
        var r0 = getInt32Memory0()[8 / 4 + 0];
        var r1 = getInt32Memory0()[8 / 4 + 1];
        var v0 = getArrayU8FromWasm0(r0, r1).slice();
        wasm.__wbindgen_free(r0, r1 * 1);
        return v0;
    }
}
/**
*/
export class Cpu {

    static __wrap(ptr) {
        const obj = Object.create(Cpu.prototype);
        obj.ptr = ptr;

        return obj;
    }

    free() {
        const ptr = this.ptr;
        this.ptr = 0;

        wasm.__wbg_cpu_free(ptr);
    }
    /**
    * @returns {Cpu}
    */
    static new() {
        var ret = wasm.cpu_new();
        return Cpu.__wrap(ret);
    }
    /**
    * @param {Cartridge} program
    */
    load_cartridge(program) {
        _assertClass(program, Cartridge);
        var ptr0 = program.ptr;
        program.ptr = 0;
        wasm.cpu_load_cartridge(this.ptr, ptr0);
    }
    /**
    */
    reset() {
        wasm.cpu_reset(this.ptr);
    }
    /**
    * @returns {ExecutionResult}
    */
    execute_cycle() {
        var ret = wasm.cpu_execute_cycle(this.ptr);
        return ExecutionResult.__wrap(ret);
    }
}
/**
*/
export class ExecutionResult {

    static __wrap(ptr) {
        const obj = Object.create(ExecutionResult.prototype);
        obj.ptr = ptr;

        return obj;
    }

    free() {
        const ptr = this.ptr;
        this.ptr = 0;

        wasm.__wbg_executionresult_free(ptr);
    }
    /**
    * @param {Uint8Array} display_state
    * @param {boolean} should_beep
    * @returns {ExecutionResult}
    */
    static new(display_state, should_beep) {
        var ptr0 = passArray8ToWasm0(display_state, wasm.__wbindgen_malloc);
        var len0 = WASM_VECTOR_LEN;
        var ret = wasm.executionresult_new(ptr0, len0, should_beep);
        return ExecutionResult.__wrap(ret);
    }
    /**
    * @returns {Uint8Array}
    */
    get_display_state() {
        wasm.executionresult_get_display_state(8, this.ptr);
        var r0 = getInt32Memory0()[8 / 4 + 0];
        var r1 = getInt32Memory0()[8 / 4 + 1];
        var v0 = getArrayU8FromWasm0(r0, r1).slice();
        wasm.__wbindgen_free(r0, r1 * 1);
        return v0;
    }
    /**
    * @returns {boolean}
    */
    get_should_beep() {
        var ret = wasm.executionresult_get_should_beep(this.ptr);
        return ret !== 0;
    }
}

async function load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {

        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);

            } catch (e) {
                if (module.headers.get('Content-Type') != 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else {
                    throw e;
                }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);

    } else {

        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };

        } else {
            return instance;
        }
    }
}

async function init(input) {
    if (typeof input === 'undefined') {
        input = import.meta.url.replace(/\.js$/, '_bg.wasm');
    }
    const imports = {};
    imports.wbg = {};
    imports.wbg.__wbindgen_throw = function(arg0, arg1) {
        throw new Error(getStringFromWasm0(arg0, arg1));
    };

    if (typeof input === 'string' || (typeof Request === 'function' && input instanceof Request) || (typeof URL === 'function' && input instanceof URL)) {
        input = fetch(input);
    }

    const { instance, module } = await load(await input, imports);

    wasm = instance.exports;
    init.__wbindgen_wasm_module = module;

    return wasm;
}

export default init;

