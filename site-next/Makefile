CLANG ?= clang

.PHONY: wasm build test serve clean

wasm:
	$(CLANG) --target=wasm32 -Oz -nostdlib -fno-builtin \
		-Wl,--no-entry -Wl,--export=overlap_count -Wl,--strip-all \
		-o assets/experiments/bitset-overlap.wasm wasm/bitset-overlap.c

build: wasm
	cargo run --release

test:
	cargo test

serve:
	python3 -m http.server 8080 --directory dist

clean:
	rm -rf dist
