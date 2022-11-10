# gst-example-rs

GstremerをRustで書くサンプル

## crate

|directory|desctiption|
|---|---|
|meta/example-rs|カスタムメタデータ `ExampleRsMeta` のRust実装|
|meta/example-rs-sys|カスタムメタデータ `ExampleRsMeta` をRustから操作するbinding|
|plugin/src/testtrans|BaseTransformを使った実装|
|plugin/src/metatrans|`ExampleRsMeta`の追加、表示などを行う実装|


## References

- [gstreamer-rs](https://gitlab.freedesktop.org/gstreamer/gstreamer-rs)
- [gst-plugins-rs](https://gitlab.freedesktop.org/gstreamer/gst-plugins-rs)
- [SRT file format](https://docs.fileformat.com/video/srt/)
