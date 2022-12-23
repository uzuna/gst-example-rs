include ./common.mk
include ./rust.mk

ELEM:=  # inspectで特定のelementを指すための変数

# plubinのビルド
.PHONY: build
build:
	cargo build ${BUILD_FLAG}

# pluginの表示
.PHONY: inspect
inspect: build
	gst-inspect-1.0 --gst-plugin-path=${RUST_OUT_DIR} ${RUST_OUT_DIR}libgstrsexample.so ${ELEM}

# gstappを起動
.PHONY: run.app
run.app: build
	cargo run

# transformのふるまいを確認
.PHONY: launch
launch: build
	GST_DEBUG=1,simpletrans:7 gst-launch-1.0 --gst-plugin-path=${RUST_OUT_DIR} videotestsrc num-buffers=30 ! simpletrans ! autovideosink
