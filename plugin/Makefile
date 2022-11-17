include ../common.mk
include ../rust.mk

BASE_VERSION?=$(shell cat ./Cargo.toml | grep -e "^version" | sed -r 's/.+"(.+)"/\1/')
ARCH:=amd64
DEB_VERSION=${BASE_VERSION}

.PHONY: all
all:
	make build TARGET=release
	make deb

.PHONY: build
build: ${RUST_OUT_DIR}/libgstrsexample.so ${BUILD_DIR}
ifeq (${TARGET}, release)
	cp ${RUST_OUT_DIR}/libexample_rs_meta.so ${BUILD_DIR}/
	cp ${RUST_OUT_DIR}/libexample_c_meta.so ${BUILD_DIR}/
	cp ${RUST_OUT_DIR}/libgstrsexample.so ${BUILD_DIR}/
endif


# メインのプラグインを生成する
${RUST_OUT_DIR}/libgstrsexample.so: ${RUST_OUT_DIR}/libexample_rs_meta.so ${RUST_OUT_DIR}/libexample_c_meta.so
	cargo build ${BUILD_FLAG}

# metadataのSOを生成するのでメインプラグインよりも先に生成する
${RUST_OUT_DIR}/libexample_rs_meta.so: ${PROJECT_DIR}meta/example-rs/src
	cd ${PROJECT_DIR}meta/example-rs && cargo build ${BUILD_FLAG}

# metadataのSOを生成するのでメインmakeプラグインよりも先に生成する
${RUST_OUT_DIR}/libexample_c_meta.so: ${PROJECT_DIR}meta/example-c
	make -C ${PROJECT_DIR}meta/example-c

.PHONY: clean
clean:
	rm -rf pkg-build dist
	rm -f ${RUST_OUT_DIR}/libgstrsexample.so
	rm -rf ${BUILD_DIR}

.PHONY: test
test:
	cargo test

.PHONY: deb
deb: ${BUILD_DIR}
	rm -rf pkg-build dist
	mkdir -p ${DEB_DIR}
	cp -r ${BUILD_DIR} .
	# need go-bin-deb: `go install github.com/mh-cbon/go-bin-deb@latest`
	go-bin-deb generate --arch ${ARCH} --version ${DEB_VERSION} --output ${DEB_DIR}
	rm -r ${ARTIFACTS_DIR_NAME}

.PHONY: cargo-deb
cargo-deb:
	make build TARGET=release
	# need cargo-deb: `cargo install cargo-deb`
	cargo deb --no-build -o ../deb/
