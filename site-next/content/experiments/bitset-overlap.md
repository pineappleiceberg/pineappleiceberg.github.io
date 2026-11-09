+++
title = "Tiny C/WebAssembly experiment: bitset intersection"
description = "A minimal, lazy-loaded C-to-WebAssembly bitset overlap experiment, demonstrating computation without client-side page rendering."
permalink = "/experiments/bitset-overlap/"
script = "/assets/experiments/bitset-overlap.js"
+++

# Bitset intersection in the browser

This experiment demonstrates how the site can keep normal articles as **static HTML**, while making an isolated computation interactive on demand.

Two unsigned 16-bit integers encode sets of bit positions. Their bitwise AND contains the positions present in both sets; the C function counts those shared set bits.

**Important:** This is *not* private set intersection. The entered values are visible to the browser and are passed directly to the WASM module. It makes no privacy or cryptographic security claim.

<div class="lab">
<form id="overlap-form">
<label for="mask-a">Alice's 16-bit mask (decimal, 0–65535)</label>
<input id="mask-a" type="number" value="15" min="0" max="65535" required>
<label for="mask-b">Bob's 16-bit mask (decimal, 0–65535)</label>
<input id="mask-b" type="number" value="10" min="0" max="65535" required>
<button type="submit">Run C in WebAssembly</button>
<output id="overlap-result" aria-live="polite">Choose values and run the experiment.</output>
</form>
<p class="note">The WASM file is fetched only when you press the button. This article is readable without JavaScript.</p>
</div>

## C implementation

The linked [source on the prototype branch](https://github.com/pineappleiceberg/pineappleiceberg.github.io/tree/prototype/native-static-wasm-site-20261109/site-next/wasm) uses a small bit-counting kernel. The compiled core has no imports and does not need a JavaScript framework or server API.

Next, this pattern can host a real cryptographic primitive or benchmark, after a careful threat model and reproducibility review.
