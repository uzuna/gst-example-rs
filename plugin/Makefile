include ../common.mk
include ../rust.mk

BASE_VERSION?=$(shell cat ./Cargo.toml | grep -e "^version" | sed -r 's/.+"(.+)"/\1/')
ARCH:=amd64
DEB_VERSION=${BASE_VERSION}

.PHONY: build
build: ${OUT_DIR}/libgstrsexample.so

# メインのプラグインを生成する
${OUT_DIR}/libgstrsexample.so: ${OUT_DIR}/libexample_rs_meta.so ${OUT_DIR}/libexample_c_meta.so
	cargo build ${BUILD_FLAG}

# metadataのSOを生成するのでメインプラグインよりも先に生成する
${OUT_DIR}/libexample_rs_meta.so: ${PROJECT_DIR}meta/example-rs/src
	cd ${PROJECT_DIR}meta/example-rs && cargo build ${BUILD_FLAG}

# metadataのSOを生成するのでメインmakeプラグインよりも先に生成する
${OUT_DIR}/libexample_c_meta.so: ${PROJECT_DIR}meta/example-c
	make -C ${PROJECT_DIR}meta/example-c

clean:
	rm -rf pkg-build dist
	rm ${OUT_DIR}/libgstrsexample.so

test:
	cargo test

deb:
	make build TARGET=release
	rm -rf pkg-build dist
	mkdir -p ${DEB_DIR}
	# need go-bin-deb: `go install github.com/mh-cbon/go-bin-deb@latest`
	go-bin-deb generate --arch ${ARCH} --version ${DEB_VERSION} --output ${DEB_DIR}

cargo-deb:
	make build TARGET=release
	# need cargo-deb: `cargo install cargo-deb`
	cargo deb --no-build -o ../deb/
