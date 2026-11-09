// This module is downloaded only on the optional experiment page.
// The WebAssembly binary is fetched only after a visitor submits the form.
const form = document.querySelector("#overlap-form");
const answer = document.querySelector("#overlap-result");
let enginePromise;

async function loadEngine() {
  const url = "/assets/experiments/bitset-overlap.wasm";
  const response = await fetch(url);
  if (!response.ok) throw new Error("WASM fetch failed: HTTP " + response.status);
  if (WebAssembly.instantiateStreaming) {
    try {
      const loaded = await WebAssembly.instantiateStreaming(response.clone(), {});
      return loaded.instance.exports;
    } catch (_) {
      // Some local servers send application/octet-stream instead of application/wasm.
    }
  }
  const loaded = await WebAssembly.instantiate(await response.arrayBuffer(), {});
  return loaded.instance.exports;
}

form?.addEventListener("submit", async (event) => {
  event.preventDefault();
  const a = Number(document.querySelector("#mask-a").value);
  const b = Number(document.querySelector("#mask-b").value);
  if (!Number.isInteger(a) || !Number.isInteger(b) || a < 0 || b < 0 || a > 65535 || b > 65535) {
    answer.textContent = "Enter integers from 0 to 65535.";
    return;
  }
  answer.textContent = "Running…";
  try {
    enginePromise ??= loadEngine();
    const engine = await enginePromise;
    const count = engine.overlap_count(a, b);
    answer.textContent = "Shared set bits: " + count + " (computed by C → WASM)";
  } catch (error) {
    enginePromise = undefined;
    answer.textContent = "The optional experiment could not load. The article remains readable.";
    console.error(error);
  }
});
