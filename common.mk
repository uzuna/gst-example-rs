# 共通の環境変数設定
mkfile_path := $(abspath $(lastword $(MAKEFILE_LIST)))
PROJECT_DIR := $(dir $(mkfile_path))
# ビルド成果物の保存先ディレクトリ名
ARTIFACTS_DIR_NAME := artifacts
# debpackage生成先ディレクトリ
DEB_DIR := ${PROJECT_DIR}deb
# ビルド成果物の絶対パス
BUILD_DIR := ${PROJECT_DIR}${ARTIFACTS_DIR_NAME}

# Rustのビルドターゲット {<empty>(dev), production}
TARGET:=

# ビルドフラグとGstreamer実行の参照先の切り替え
ifeq (${TARGET}, release)
	BUILD_FLAG=--release
# 出力先をrustのビルドディレクトリとする
	RUST_OUT_DIR=${PROJECT_DIR}target/release/
else
	BUILD_FLAG=
	RUST_OUT_DIR=${PROJECT_DIR}target/debug/
endif

${BUILD_DIR}:
	mkdir -p ${BUILD_DIR}

clean:
	rm -rf ${BUILD_DIR}
	rm -rf ${DEB_DIR}
	cargo clean
