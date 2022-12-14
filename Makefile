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

# klvsrcをtsmuxに入れてファイルに保存
.PHONY: run.save_klvts
run.save_klvts: build
	LD_LIBRARY_PATH=${RUST_OUT_DIR} GST_DEBUG_FILE=gst.log GST_DEBUG=3,klvtestsrc:7,mpegtsmux:7 gst-launch-1.0 --gst-plugin-path=${RUST_OUT_DIR} videotestsrc is-live=true ! video/x-raw,framerate=30/1 ! x264enc ! mpegtsmux name=m ! filesink location=test.m2ts klvtestsrc is-live=true fps=10 ! m.

# m2tsファイルを再生してklvを確認
.PHONY: run.play_klvts
run.play_klvts: build
	GST_DEBUG_DUMP_DOT_DIR=. LD_LIBRARY_PATH=${RUST_OUT_DIR} GST_DEBUG=1,rsidentity:7 gst-launch-1.0 --gst-plugin-path=${RUST_OUT_DIR} filesrc location=test.m2ts ! tsdemux name=t ! h264parse ! avdec_h264 ! rsidentity ! autovideosink t. ! meta/x-klv,parsed=true ! queue ! fakesink dump=true 

# probe動作を見る
.PHONY: run.probe
run.probe: build
	LD_LIBRARY_PATH=${RUST_OUT_DIR} GST_PLUGIN_PATH=${RUST_OUT_DIR} GST_DEBUG=decodebin:7,autovideosink:7 cargo run probe --fps 5

# klv demuxをgst-launchから起動
.PHONY: run.demux
run.demux: build
# mpegtsmux -> tsdemuxの間にqueueを挟まなければ2つのストリームが認識されないためdemuxでprivate padが生えない
# だから複数bufferを流すqueueが必要と思われる
	LD_LIBRARY_PATH=${RUST_OUT_DIR} GST_DEBUG_FILE=gst.log GST_DEBUG=1,metademux:7,mpegtsmux:7 gst-launch-1.0 --gst-plugin-path=${RUST_OUT_DIR} videotestsrc num-buffers=20 ! video/x-raw,framerate=5/1 ! metatrans name=addrs op=add ! metademux name=d ! queue max-size-buffers=1 ! x264enc ! mpegtsmux name=m ! filesink location=test.m2ts d. ! queue max-size-time=0 ! m.

# klv demuxをapplicationで構築して起動
.PHONY: run.demux-app
run.demux-app: build
# launch by application
	LD_LIBRARY_PATH=${RUST_OUT_DIR} GST_PLUGIN_PATH=${RUST_OUT_DIR} GST_DEBUG=3 cargo run probe-tsdemux --fps 10

# klv muxを使う
.PHONY: run.mux
run.mux: build
# launch by application
	LD_LIBRARY_PATH=${RUST_OUT_DIR} GST_DEBUG=1,metamux:3,metatrans:7 gst-launch-1.0 --gst-plugin-path=${RUST_OUT_DIR} filesrc location=test.m2ts ! tsdemux name=t ! h264parse ! avdec_h264 ! metamux name=m ! metatrans op=show ! autovideosink t. ! meta/x-klv,parsed=true ! queue max-size-time=0 ! m.

.PHONY: deb
deb:
	make -C plugin deb
