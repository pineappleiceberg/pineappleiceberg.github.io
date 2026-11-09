# Static-first site prototype

A deliberately isolated proof of concept for amrishhallberg.com. It does not replace the Jekyll publishing branch and must not be deployed to the primary domain in this state.

Design:
- Rust is the **build-time-only** compiler, using pulldown-cmark to render trusted Markdown to semantic HTML.
- Browser pages are static HTML and CSS. Ordinary pages contain zero JavaScript.
- Experiments can opt in to a small script; C/WebAssembly loads only after user interaction.
- No browser runtime, SPA hydration, analytics trackers, or external web fonts.
- All metadata has explicit canonical URLs and escaped meta tags.
- Pages have portable, explicit permalinks, and the generator rejects unsafe paths or duplicate output targets.

## Requirements

Rust (cargo), Clang with wasm32 + wasm-ld, Make, and Python 3 for the optional local preview server.

## Build

From this directory:

    make build
    make test
    make serve

Then view http://localhost:8080/.

To verify the tiny native-to-WASM artifact:

    wc -c assets/experiments/bitset-overlap.wasm

The prototype's initial WebAssembly binary is **76 bytes**, tested locally with Node.js. It is included so the experiment also works if the WASM compiler is unavailable for local editing. The Make target regenerates it.

## Cutover constraints

1. Keep the existing site unchanged until a full migration has been reviewed.
2. Preserve the three published article URLs byte-for-byte, not just their slugs.
3. Migrate each complete technical article, including code samples, screenshots, image credits, equations, and internal links. Don't reduce completeness for smaller HTML.
4. Compare uncompressed and Brotli/gzip HTML, DOM nodes, total images, and Core Web Vitals before/after on representative pages.
5. Maintain metadata, canonical URLs, social preview tags, accessible navigation, sitemap, RSS, redirects if genuinely necessary, and a tested deployment/rollback path.
6. Add a separate build-only GitHub Actions job before enabling GitHub Pages publishing from the new toolchain.
7. Do not merge the earlier Jekyll SEO draft PR blindly; transplant its applicable copy and metadata into this new architecture.

## Not implemented

Full migration of old articles, build-time syntax highlighting, build-time math, responsive image conversions, RSS/sitemap generation, performance measurements, or production Pages deployment. These are deliberate follow-up milestones.

The experimental page models an ordinary bitset intersection, not cryptographic private set intersection.
