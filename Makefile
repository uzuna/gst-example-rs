

ELEM:=  # inspectで特定のelementを指すための変数
COPYMODE:=meta  # run.transのコピーモード動作指示
TMETHOD:=copy  # run.metaのメタデータtransform動作の指示
TARGET:=  # Rustのビルドターゲット {<empty>(dev), production}

# ビルドフラグとGstreamer実行の参照先の切り替え
ifeq (${TARGET}, release)
	BUILD_FLAG=--release
	OUT_DIR=target/release
else
	BUILD_FLAG=
	OUT_DIR=target/debug
endif

# 全体buildのエントリポイント
.PHONY: build
build: ${OUT_DIR}/libgstrsexample.so
	cargo build ${BUILD_FLAG}

# メインのプラグインを生成する
${OUT_DIR}/libgstrsexample.so: plugin/src/ ${OUT_DIR}/libexample_rs_meta.so ${OUT_DIR}/libexample_c_meta.so
	cd plugin && cargo build ${BUILD_FLAG}

# metadataのSOを生成するのでメインプラグインよりも先に生成する
${OUT_DIR}/libexample_rs_meta.so: meta/example-rs/src
	cd meta/example-rs && cargo build ${BUILD_FLAG}

# metadataのSOを生成するのでメインプラグインよりも先に生成する
${OUT_DIR}/libexample_c_meta.so: meta/example-c
	make -C meta/example-c

# pluginの表示
.PHONY: inspect
inspect: build
	LD_LIBRARY_PATH=${OUT_DIR} gst-inspect-1.0 --gst-plugin-path=${OUT_DIR} ${OUT_DIR}/libgstrsexample.so ${ELEM}

# gstappを起動
.PHONY: run.app
run.app: build
	cargo run

# transformのふるまいを確認
.PHONY: run.trans
run.trans: build
	LD_LIBRARY_PATH=${OUT_DIR} GST_DEBUG=1,testtrans:7 gst-launch-1.0 --gst-plugin-path=${OUT_DIR} videotestsrc ! testtrans copymode=${COPYMODE} ! autovideosink

# metadataの付与と表示テスト
.PHONY: run.meta
run.meta: build
	LD_LIBRARY_PATH=${OUT_DIR} GST_DEBUG=1,metatrans:7 gst-launch-1.0 --gst-plugin-path=${OUT_DIR} videotestsrc ! video/x-raw,width=600,height=400,framerate=300/1 ! metatrans name=addrs op=add tmethod=${TMETHOD} ! metatrans name=addc mtype=c op=add tmethod=${TMETHOD} ! metatrans name=fc op=show mtype=c ! videoscale ! video/x-raw,width=300,height=200 ! videoconvert ! testtrans copymode=${COPYMODE} ! metatrans name=frs op=show ! metatrans op=remove ! metatrans op=show ! autovideosink

.PHONY: fmt
fmt:
	cargo fmt
	cargo clippy --fix --allow-staged

.PHONY: check-fmt
check-fmt:
	cargo fmt --check
	cargo clippy
