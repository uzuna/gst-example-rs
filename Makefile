
ARG:=
COPYMODE:=meta
OUT_DIR=target/debug

.PHONY: build
build: ${OUT_DIR}/libgstrsexample.so plugin/src
	cargo build
${OUT_DIR}/libgstrsexample.so: plugin/src/
	cargo build

.PHONY: inspect
inspect: build
	LD_LIBRARY_PATH=${OUT_DIR} gst-inspect-1.0 --gst-plugin-path=${OUT_DIR} ${OUT_DIR}/libgstrsexample.so ${ARG}

.PHONY: run.trans
run.trans: build
	LD_LIBRARY_PATH=${OUT_DIR} GST_DEBUG=1,testtrans:7 gst-launch-1.0 --gst-plugin-path=${OUT_DIR} videotestsrc ! testtrans copymode=${COPYMODE} ! autovideosink

.PHONY: run.meta
run.meta: build
	LD_LIBRARY_PATH=${OUT_DIR} GST_DEBUG=1,metatrans:7 gst-launch-1.0 --gst-plugin-path=${OUT_DIR} videotestsrc ! metatrans op=add ! videoconvert ! metatrans op=show ! autovideosink

.PHONY: fmt
fmt:
	cargo fmt
	cargo clippy --fix --allow-staged

.PHONY: check-fmt
check-fmt:
	cargo fmt --check
	cargo clippy
