include ./common.mk
include ./rust.mk

ELEM:=  # inspectで特定のelementを指すための変数
COPYMODE:=meta  # run.transのコピーモード動作指示
TMETHOD:=copy  # run.metaのメタデータtransform動作の指示

# 全体buildのエントリポイント
.PHONY: build
build: ${RUST_OUT_DIR}/libgstrsexample.so
	cargo build ${BUILD_FLAG}

${RUST_OUT_DIR}/libgstrsexample.so:
	make -C plugin build

# pluginの表示
.PHONY: inspect
inspect: build
	LD_LIBRARY_PATH=${RUST_OUT_DIR} gst-inspect-1.0 --gst-plugin-path=${RUST_OUT_DIR} ${RUST_OUT_DIR}/libgstrsexample.so ${ELEM}

# gstappを起動
.PHONY: run.app
run.app: build
	cargo run

# transformのふるまいを確認
.PHONY: run.trans
run.trans: build
	LD_LIBRARY_PATH=${RUST_OUT_DIR} GST_DEBUG=1,testtrans:7 gst-launch-1.0 --gst-plugin-path=${RUST_OUT_DIR} videotestsrc ! testtrans copymode=${COPYMODE} ! autovideosink

# metadataの付与と表示テスト
.PHONY: run.meta
run.meta: build
	LD_LIBRARY_PATH=${RUST_OUT_DIR} GST_DEBUG=1,metatrans:7 gst-launch-1.0 --gst-plugin-path=${RUST_OUT_DIR} videotestsrc ! video/x-raw,width=600,height=400,framerate=300/1 ! metatrans name=addrs op=add tmethod=${TMETHOD} ! metatrans name=addc mtype=c op=add tmethod=${TMETHOD} ! metatrans name=fc op=show mtype=c ! videoscale ! video/x-raw,width=300,height=200 ! videoconvert ! testtrans copymode=${COPYMODE} ! metatrans name=frs op=show ! metatrans op=remove ! metatrans op=show ! autovideosink

# metadataの付与と表示テスト
.PHONY: run.bin
run.bin: build
	LD_LIBRARY_PATH=${RUST_OUT_DIR} GST_DEBUG=1,exsrcbin:7,metatrans:7 gst-launch-1.0 --gst-plugin-path=${RUST_OUT_DIR} exampletestsrc fps=10/1 ! videoscale ! video/x-raw,width=300,height=200 ! metatrans op=show ! autovideosink

# klv demux
.PHONY: run.klvsrc
run.klvsrc: build
# パイプラインが2分岐するのでqueueが必要
	LD_LIBRARY_PATH=${RUST_OUT_DIR} GST_DEBUG=1,klvtestsrc:7 gst-launch-1.0 --gst-plugin-path=${RUST_OUT_DIR} klvtestsrc ! fakesink dump=true sync=true

.PHONY: run.save_klvts
run.save_klvts: build
	LD_LIBRARY_PATH=${RUST_OUT_DIR} GST_DEBUG=3,klvtestsrc:7,filesink:7 gst-launch-1.0 --gst-plugin-path=${RUST_OUT_DIR} videotestsrc is-live=true ! video/x-raw,framerate=30/1 ! x264enc ! mpegtsmux name=m ! filesink location=test.m2ts klvtestsrc is-live=true fps=10 ! m.

.PHONY: run.play_klvts
run.play_klvts: build
	LD_LIBRARY_PATH=${RUST_OUT_DIR} GST_DEBUG=3 gst-launch-1.0 --gst-plugin-path=${RUST_OUT_DIR} filesrc location=test.m2ts ! tsdemux name=t ! h264parse ! avdec_h264 ! autovideosink t. ! meta/x-klv,parsed=true ! queue ! fakesink dump=true 

.PHONY: run.probe
run.probe: build
	LD_LIBRARY_PATH=${RUST_OUT_DIR} GST_PLUGIN_PATH=${RUST_OUT_DIR} GST_DEBUG=decodebin:7,autovideosink:7 cargo run probe --fps 5

.PHONY: run.probecmd
run.probecmd: build
	LD_LIBRARY_PATH=${RUST_OUT_DIR} GST_PLUGIN_PATH=${RUST_OUT_DIR} gst-launch-1.0 --gst-plugin-path=${RUST_OUT_DIR} videotestsrc ! video/x-raw,framerate=30/1 ! x264enc ! mpegtsmux name=m ! tsdemux ! decodebin ! autovideosink klvtestsrc fps=10 ! m.


# klv demux
.PHONY: run.demux-noenc
run.demux-noenc: build
# no encoding
	LD_LIBRARY_PATH=${RUST_OUT_DIR} GST_DEBUG_FILE=gst.log GST_DEBUG=1,metademux:7 gst-launch-1.0 --gst-plugin-path=${RUST_OUT_DIR} videotestsrc num-buffers=20 ! video/x-raw,framerate=5/1 ! metatrans name=addrs op=add ! metademux name=d ! queue ! autovideosink d. ! fakesink dump=true sync=true

# klv demux
.PHONY: run.demux
run.demux: build
# mpegtsmux -> tsdemuxの間にqueueを挟まなければ2つのストリームが認識されないためdemuxでprivate padが生えない
# だから複数bufferを流すqueueが必要と思われる
	LD_LIBRARY_PATH=${RUST_OUT_DIR} GST_DEBUG_FILE=gst.log GST_DEBUG=1,metademux:7,mpegtsmux:7 gst-launch-1.0 --gst-plugin-path=${RUST_OUT_DIR} videotestsrc num-buffers=20 ! video/x-raw,framerate=5/1 ! metatrans name=addrs op=add ! x264enc ! metademux name=d ! queue ! mpegtsmux name=m ! queue ! filesink location=test.m2ts sync=true d. ! queue ! m.

# klv demux
.PHONY: run.demux-app
run.demux-app: build
# launch by application
	RUST_LOG=debug LD_LIBRARY_PATH=${RUST_OUT_DIR} GST_PLUGIN_PATH=${RUST_OUT_DIR} GST_DEBUG=1,metademux:7 cargo run probe-tsdemux --fps 5

.PHONY: deb
deb:
	make -C plugin deb
