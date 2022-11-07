use anyhow::Error;
use gst::prelude::*;

/// textoverlayで一定の周期で変化するテキストをかぶせる実装サンプル
/// テキストはAppSrcを使って生成する
pub(crate) fn build_appsrc_text_overlay() -> Result<gst::Pipeline, Error> {
    gst::init()?;

    let pipeline = gst::Pipeline::default();
    let appsrc = gst_app::AppSrc::builder()
        .caps(
            &gst::Caps::builder("text/x-raw")
                .field("format", "pango-markup")
                .build(),
        )
        .format(gst::Format::Time)
        .build();

    let videosrc = gst::ElementFactory::make("videotestsrc").build()?;
    let textoverlay = gst::ElementFactory::make("textoverlay").build()?;
    let queue = gst::ElementFactory::make("queue").build()?;
    let sink = gst::ElementFactory::make("autovideosink").build()?;

    textoverlay.set_property("wait-text", false);

    // add to pipeline and link elements
    pipeline.add_many(&[&videosrc, &queue, appsrc.upcast_ref(), &textoverlay, &sink])?;
    gst::Element::link_many(&[&textoverlay, &sink])?;
    // textoverlayは先にvideoをlinkする。順序依存性がある
    videosrc.link(&textoverlay)?;
    gst::Element::link_many(&[appsrc.upcast_ref(), &queue, &textoverlay])?;

    // appsrc関係の処理ブロック
    {
        let mut frame_count = 0;
        let step = gst::ClockTime::SECOND;
        appsrc.set_callbacks(
            gst_app::AppSrcCallbacks::builder()
                .need_data(move |appsrc, _hint_of_byte_size| {
                    let text = format!(r#"<span foreground="white" font_desc="Sans Italic 36">frame_count: {}</span>"#, frame_count);
                    let Ok(mut buffer) = gst::Buffer::with_size(text.len()) else {
                        appsrc.abort_state();
                        return
                    };
                    {
                        let buffer = buffer.get_mut().unwrap();
                        // 表示するタイミングと長さを指定する
                        // この実装の場合はtextoverlayのvideoの周期より早くは出来ない
                        buffer.set_pts(frame_count * step);
                        buffer.set_duration(step);
                        let Ok(mut buffer_map) = buffer.map_writable() else {
                            appsrc.abort_state();
                            return
                        };
                        buffer_map.copy_from_slice(text.as_bytes());
                    }
                    frame_count += 1;

                    // need_dataのたびに1bufferをpushする
                    // 事前に生成して送りたい場合はenough_dataと組み合わせて
                    // 複数バッファを送り、停止要求を受けたら止める実装となる
                    // e.g. https://gitlab.freedesktop.org/gstreamer/gstreamer-rs/-/blob/main/tutorials/src/bin/basic-tutorial-8.rs
                    let _ = appsrc.push_buffer(buffer);
                })
                .build(),
        );
    }
    Ok(pipeline)
}
