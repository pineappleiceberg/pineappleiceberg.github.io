---
layout: post
title: "Private Set Intersection demo (C/WASM + garbled circuits)"
date: 2025-11-28
tags: [cryptography, psi, wasm, garbled-circuits, blake3]
---

**Live demo (interactive, in-browser):**  
<https://pineappleiceberg.github.io/psiweb/>

**Source code (C core + WebAssembly glue):**  
<https://github.com/pineappleiceberg/psiweb>

This project is a small, self-contained *private set intersection* (PSI) core
written in C and compiled to WebAssembly, with a browser UI on top. The page
compares two PSI flavours that share the same underlying C library:

- a **hash-only** PSI (naive \(O(n^2)\) memcmp over fixed-size digests), and  
- a **GC-backed** PSI that uses a Yao garbled-circuit equality core for each
  pair of digests.

Because both PSI implementations operate on the *same* hashed inputs, the
JavaScript layer can check that their masks match for every run and compare
their runtimes side by side.

---

## Set model and keyed hashing

Let Alice and Bob each hold a finite set of strings
\(A = \{a_0,\dots,a_{n-1}\}\) and \(B = \{b_0,\dots,b_{n-1}\}\).  
We want an indicator mask \(m \in \{0,1\}^n\) such that

\[
m_i =
\begin{cases}
1 & \text{if } \exists j : a_i = b_j, \\[4pt]
0 & \text{otherwise.}
\end{cases}
\]

The intersection is then

\[
I = \{ a_i \mid m_i = 1 \}.
\]

Instead of comparing raw strings, the C core maps each element into a fixed
digest space using **keyed BLAKE3** in PRF/MAC mode, truncated to 16 bytes
(128 bits):

```text
H_k(s) = BLAKE3_keyed(k, s)[0..15]     // 16-byte digest

A' = { H_k(a_0), …, H_k(a_{n-1}) }
B' = { H_k(b_0), …, H_k(b_{n-1}) }
```

In the **hash-only** PSI, the WebAssembly module simply checks

\[
m_i = \bigvee_{j=0}^{n-1} [\, H_k(a_i) = H_k(b_j) \,],
\]

using a naive nested loop and a byte-wise `memcmp` over the flat digest arrays.

> **Note.** In this *demo*, the BLAKE3 key is a fixed constant compiled into
> the module for reproducibility. In a real deployment it would be a shared,
> secret key (and preferably per-session), so that an attacker who can read the
> code cannot mount trivial offline dictionary attacks. Because of this, if you use the code at the root of this project, please note you need an OT protcol and (likely) networking of some kind. 

---

## Bit-level equality circuit

For the GC-backed path, each digest is treated as a length-\(k\) bit vector.
Let

\[
A_i = (a_i^{(0)},\dots,a_i^{(k-1)}), \quad
B_j = (b_j^{(0)},\dots,b_j^{(k-1)}),
\]

where typically \(k = 8 \times 16 = 128\) for a 16-byte digest.

For each pair \((A_i,B_j)\), the Boolean circuit computes a single equality
bit \(\mathsf{eq}(A_i,B_j)\) according to:

```text
// per-bit XOR and equality
x_t = a_i^(t) XOR b_j^(t)        for t = 0,…,k-1
e_t = NOT(x_t)                   // e_t = 1 iff bits match

// full equality bit
eq  = e_0 AND e_1 AND … AND e_(k-1)
```

So the PSI mask can be written as

\[
m_i = \bigvee_{j=0}^{n-1} \mathsf{eq}(A'_i, B'_j),
\]

where `eq` is this equality circuit applied to the digests \(A'_i, B'_j\).

In the implementation, this equality circuit is built once for a given
`elem_bits` and then reused for every \((i,j)\) pair. The GC engine
instantiates it using AND, XOR, and NOT gates on top of wire labels.

---

## Yao garbled circuits with Free-XOR

The PSI demo uses a standard **Yao garbled-circuit** backend with the
**Free-XOR** optimization:

- Every wire \(w\) has two labels (random-looking bitstrings)
  \(L_w^0, L_w^1\) representing logical 0 and 1.  
- A single global offset \(\Delta\) is sampled once with the least
  significant bit set, and the invariant
  \(L_w^1 = L_w^0 \oplus \Delta\) is enforced for all wires.  
- The least significant bit (LSB) of each label is used as a
  **permute bit** to index into a garbled table.

Conceptually:

```text
// Label relationship for every wire w
L_w^1 = L_w^0 XOR Δ     // Δ is global with LSB(Δ) = 1

// permute bit = LSB of first byte, used as row index bit
permute_bit(L_w^b) = L_w^b[0] & 1
```

For XOR gates, this makes garbling *free*; the garbler just sets

\[
L_{out}^0 = L_{in0}^0 \oplus L_{in1}^0, \quad
L_{out}^1 = L_{out}^0 \oplus \Delta,
\]

and the evaluator XORs the input labels to obtain the output label.
No ciphertexts are needed for XOR gates at all.

Non-linear gates (AND, NOT) still require a small encrypted table.

---

## Gate encryption with keyed BLAKE3

For each non-XOR gate with inputs on wires \(u,v\) and output wire \(w\):

1. The garbler fixes the output labels \(L_w^0, L_w^1\) according to the
   `L^1 = L^0 XOR Δ` rule.  
2. For each input bit pair \((a,b) \in \{0,1\}^2\), it chooses the correct
   output label
   \(K_{out} = L_w^{f(a,b)}\), where \(f\) is the gate’s truth table
   (AND, NOT, etc.).  
3. It forms row index

   ```text
   row = (permute_bit(K_a) << 1) | permute_bit(K_b)
   ```

   where \(K_a\) and \(K_b\) are the input labels for bits \(a,b\).  
4. It derives a keystream using **keyed BLAKE3** as a pseudorandom function:

   ```text
   keystream = BLAKE3_keyed(GC_PRF_KEY, K_a || K_b || gate_index || row)
   ```

5. It stores the ciphertext row

   ```text
   table[row] = K_out XOR keystream
   ```

At evaluation time, the evaluator holds active input labels \(K_a, K_b\) for a
gate and proceeds as follows:

1. Compute `row` from the permute bits of \(K_a, K_b\).  
2. Recompute the same keystream with the shared `GC_PRF_KEY`.  
3. XOR the table entry with the keystream to recover the unique output label
   \(K_{out}\).

Because exactly one row per gate is ever decrypted, using a one-time-pad style
XOR with a PRF keystream is sufficient for the standard Yao security argument here.

---

## Browser architecture and data flow

The WebAssembly module exports two main PSI entry points:

- `psi_hash_only_compute` – runs the naive hash-only PSI over digests.  
- `psi_gc_compute` – runs the GC-backed equality PSI over the same digests.

The browser UI:

1. Read two newline-separated lists of strings from the textareas
   (“Alice’s set” and “Bob’s set”).  
2. Marshal them into contiguous buffers and hand them to the WASM module.  
3. Let the C core hash all strings with keyed BLAKE3 into fixed-size digests.  
4. Run **both** PSI variants on those digests:
   - record the time and intersection size for the hash-only variant,  
   - record the time and intersection size for the GC-backed variant.  
5. Cross-check that both variants return the same mask, and show any mismatch
   as an error.  
6. Present the timings and intersection size in the UI.

This setup keeps all interesting work inside a small, testable C library, while
the JavaScript layer is only responsible for IO, display, and light
marshalling.

---

## Security model

In this browser demo, **both parties’ inputs live in the same process** (one
page, one WASM instance). The goal is:

- to exercise the PSI core,
- to compare correctness and performance of hash-only vs GC-backed PSI, and
- to provide a concrete, inspectable implementation of a Yao GC equality
  circuit with keyed BLAKE3 as a PRF.

It is *not* a complete two-party private PSI protocol by itself.

A full PSI protocol would:

- run Alice and Bob on **separate machines / processes**,  
- use **oblivious transfer (OT)** or equivalent techniques so that Bob can
  obtain the input labels for his bits without revealing those bits to the
  garbler, and  
- send the garbled circuit and Alice’s input labels over the network, so that
  only the intersection (and nothing else) is revealed to the parties in the
  semi-honest model.

Implementing that full protocol would add more complexity in the WASM environment (OT extension,
networking, malicious-security checks, etc.), but the **equality circuit and
GC engine in this demo are already suitable as the cryptographic core** of such
a PSI.

---

## How to use the demo

1. Open the live demo in a modern browser:  
   <https://pineappleiceberg.github.io/psiweb/>
2. Enter or generate newline-separated strings into the Alice and Bob text areas.  
3. Click **“Run PSI (hash-only & GC)”**.  
4. Inspect the reported runtimes and intersection size for both variants.  
5. Optionally, open the browser devtools console to see any internal checks or
   messages if you deliberately inject mismatched data or exotic inputs.

The code is deliberately small and direct, so it can be read end-to-end as an example of PSI, garbled circuits, and WebAssembly integration.

