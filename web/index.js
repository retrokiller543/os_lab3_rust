import { WASI } from "https://cdn.skypack.dev/@wasmer/wasi";
import { WasmFs } from "https://cdn.skypack.dev/@wasmer/wasmfs";
import { lowerI64Imports } from "https://cdn.skypack.dev/@wasmer/wasm-transformer";

// Initialize the custom filesystem
const wasmFs = new WasmFs();

// Function to append text to the HTML element
function appendToBody(text) {
    const element = document.getElementById("wasm-output");
    element.textContent += text + "\n"; // Add new text as a new line
}

// Redirect stdout
wasmFs.fs.writeSync = (fd, buffer) => {
    // Assuming fd 1 is stdout
    if (fd === 1) {
        const text = new TextDecoder("utf-8").decode(buffer);
        appendToBody(text);
    }
    return buffer.length;
};

let wasi = new WASI({
    fs: wasmFs.fs,
    // Other configurations...
});

(async module => {
    const response = await fetch("os_lab3.wasm");
    const bytes = await response.arrayBuffer();
    const { instance } = await WebAssembly.instantiate(bytes, {
        ...(wasi.getImports(module)),
    });

    wasi.start(instance);
})();


// "../target/wasm32-wasi/release/os_lab3.wasm"