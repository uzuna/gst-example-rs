
ARG:=
COPYMODE:=meta
TMETHOD:=copy
OUT_DIR=target/debug

# build向けのアンカー
.PHONY: build
build: ${OUT_DIR}/libgstrsexample.so plugin/src ${OUT_DIR}/libexample_rs_meta.so meta/example-rs/src meta/example-rs-sys/src
	cargo build

# メインのプラグインを生成する
${OUT_DIR}/libgstrsexample.so: plugin/src/
	cargo build

# metadataのSOを生成するのでメインプラグインよりも先に生成する
${OUT_DIR}/libexample_rs_meta.so: meta/example-rs/src
	cd meta && cargo build

# pluginの表示
.PHONY: inspect
inspect: build
	LD_LIBRARY_PATH=${OUT_DIR} gst-inspect-1.0 --gst-plugin-path=${OUT_DIR} ${OUT_DIR}/libgstrsexample.so ${ARG}

# transformのふるまいを確認
.PHONY: run.trans
run.trans: build
	LD_LIBRARY_PATH=${OUT_DIR} GST_DEBUG=1,testtrans:7 gst-launch-1.0 --gst-plugin-path=${OUT_DIR} videotestsrc ! testtrans copymode=${COPYMODE} ! autovideosink

# metadataの付与と表示テスト
.PHONY: run.meta
run.meta: build
	LD_LIBRARY_PATH=${OUT_DIR} GST_DEBUG=1,metatrans:7 gst-launch-1.0 --gst-plugin-path=${OUT_DIR} videotestsrc ! video/x-raw,width=600,height=400 ! metatrans op=add tmethod=${TMETHOD} ! videoscale ! video/x-raw,width=300,height=200 ! videoconvert ! testtrans copymode=${COPYMODE} ! metatrans op=show ! autovideosink

.PHONY: fmt
fmt:
	cargo fmt
	cargo clippy --fix --allow-staged

.PHONY: check-fmt
check-fmt:
	cargo fmt --check
	cargo clippy
