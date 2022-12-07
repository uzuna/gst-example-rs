# 作業メモ

## TeeのあとにQueueが必要な理由を探る

複数の出力がしたい場合にteeを使って分岐するが後段にqueueが必要となる。
どのようにそれが動いているのかをProbeやlogを使って確認する。

パイプラインは以下の構造となっており `--usequeue` でqueueの有無を切り替えている

1. videotestsrc(v_src)
2. tee
    3. (queue)? autovideosink(sink1)
    4. autovideosink(sink2)

```sh
RUST_LOG=debug GST_DEBUG=xvimagesink:7 cargo run probe-tee --num-buffers 3
RUST_LOG=debug GST_DEBUG=xvimagesink:7 cargo run probe-tee --num-buffers 3 --usequeue
```

### 結果

queueなしログを見るとsrcからsink1までのイベントがトリガーされて止まっている。
tee2つ目のsrcが駆動されておらずsink2がPAUSEDにならない、するとパイプラインがPAUSEDにならないためPREROLLが終わらずPLAYINGにならない。
分岐は深さ方向に進んだあとに横に展開するものと考えて組み立てる必要がありそう

#### qurue なし

```log
[INFO  gst_example_rs::probe] v_src buffer_offset: 0
[INFO  gst_example_rs::probe] tee_sink buffer_offset: 0
[INFO  gst_example_rs::probe] tee_src1 buffer_offset: 0
[INFO  gst_example_rs::probe] sinkpad1 buffer_offset: 0
0:00:00.044123836 93115 0x5628de26e300 LOG              xvimagesink xvimagesink.c:949:gst_xv_image_sink_show_frame:<autovideosink0-actual-sink-xvimage> buffer 0x5628de2b15a0 not from our pool, copying
0:00:00.044786392 93115 0x5628de26e300 LOG              xvimagesink xvimagesink.c:262:gst_xv_image_sink_xvimage_put:<autovideosink0-actual-sink-xvimage> reffing 0x5628de2b1900 as our current image
```

#### queue あり

```log
[INFO  gst_example_rs::probe] v_src buffer_offset: 0
[INFO  gst_example_rs::probe] tee_sink buffer_offset: 0
[INFO  gst_example_rs::probe] tee_src1 buffer_offset: 0
[INFO  gst_example_rs::probe] q1_sink buffer_offset: 0
[INFO  gst_example_rs::probe] tee_src2 buffer_offset: 0
[INFO  gst_example_rs::probe] sinkpad2 buffer_offset: 0
[INFO  gst_example_rs::probe] q1_src buffer_offset: 0
[INFO  gst_example_rs::probe] sinkpad1 buffer_offset: 0
0:00:00.059771356 92816 0x561b832c7b60 LOG              xvimagesink xvimagesink.c:949:gst_xv_image_sink_show_frame:<autovideosink1-actual-sink-xvimage> buffer 0x561b833005a0 not from our pool, copying
0:00:00.059774506 92816 0x561b832c7b00 LOG              xvimagesink xvimagesink.c:949:gst_xv_image_sink_show_frame:<autovideosink0-actual-sink-xvimage> buffer 0x561b833005a0 not from our pool, copying
0:00:00.065288644 92816 0x561b832c7b60 LOG              xvimagesink xvimagesink.c:262:gst_xv_image_sink_xvimage_put:<autovideosink1-actual-sink-xvimage> reffing 0x561b83300900 as our current image
0:00:00.065287084 92816 0x561b832c7b00 LOG        
```
