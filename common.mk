# 共通の環境変数設定
mkfile_path := $(abspath $(lastword $(MAKEFILE_LIST)))
PROJECT_DIR := $(dir $(mkfile_path))

TARGET:=  # Rustのビルドターゲット {<empty>(dev), production}

# ビルドフラグとGstreamer実行の参照先の切り替え
ifeq (${TARGET}, release)
	BUILD_FLAG=--release
	OUT_DIR=${PROJECT_DIR}target/release # 出力先をrustのビルドディレクトリとする
else
	BUILD_FLAG=
	OUT_DIR=${PROJECT_DIR}target/debug
endif
