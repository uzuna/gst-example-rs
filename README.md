# gst-example-rs

GstremerをRustで書くサンプル

## crate

|directory|desctiption|
|---|---|
|meta/example-rs|カスタムメタデータ `ExampleRsMeta` のRust実装|
|meta/example-rs-sys|カスタムメタデータ `ExampleRsMeta` をRustから操作するbinding|
|plugin|Plugin実装。主にBaseTransformを使った実装例|
|gst-example|gst-launchもしくはGstAppを用いたアプリケーション実装例|

## References

- [gstreamer-rs](https://gitlab.freedesktop.org/gstreamer/gstreamer-rs)
- [gst-plugins-rs](https://gitlab.freedesktop.org/gstreamer/gst-plugins-rs)
- [SRT file format](https://docs.fileformat.com/video/srt/)
