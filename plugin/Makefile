include ../common.mk
include ../rust.mk

.PHONY: build
build: ${OUT_DIR}/libgstrsexample.so

# メインのプラグインを生成する
${OUT_DIR}/libgstrsexample.so: src/ ${OUT_DIR}/libexample_rs_meta.so ${OUT_DIR}/libexample_c_meta.so
	cargo build ${BUILD_FLAG}

# metadataのSOを生成するのでメインプラグインよりも先に生成する
${OUT_DIR}/libexample_rs_meta.so: ${PROJECT_DIR}meta/example-rs/src
	cd ${PROJECT_DIR}meta/example-rs && cargo build ${BUILD_FLAG}

# metadataのSOを生成するのでメインプラグインよりも先に生成する
${OUT_DIR}/libexample_c_meta.so: ${PROJECT_DIR}meta/example-c
	make -C ${PROJECT_DIR}meta/example-c

clean:
	rm ${OUT_DIR}/libgstrsexample.so

test:
	cargo test