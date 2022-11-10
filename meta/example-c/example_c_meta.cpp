#include "example_c_meta.h"

/*
API Typeの登録。
GLibにメタデータの存在を通知して参照のためのポインタを確保する
*/
GType example_c_meta_api_get_type(void)
{
    static volatile GType type;
    static const gchar *tags[] = {NULL};

    if (g_once_init_enter(&type))
    {
        GType _type = gst_meta_api_type_register("ExampleCMetaAPI", tags);
        g_once_init_leave(&type, _type);
    }
    return type;
}

/*
Metaデータの初期化関数
*/
gboolean
gst_example_c_meta_init(GstMeta *meta, gpointer params, GstBuffer *buffer)
{
    _ExampleCMeta *dmeta = (_ExampleCMeta *)meta;

    dmeta->count = 0;
    dmeta->num = 0.0;
    return TRUE;
}

/*
Metaデータ破棄時の処理
Heap領域を使う情報の場合はここで開放をする
*/
void gst_example_c_meta_free(GstMeta *meta, GstBuffer *buffer)
{
}

/*
Metaデータを異なるBufferに移動する時の関数
*/
static gboolean
gst_example_c_meta_transform(GstBuffer *dest, GstMeta *meta,
                             GstBuffer *buffer, GQuark type, gpointer data)
{
    _ExampleCMeta *dmeta, *smeta;
    smeta = (_ExampleCMeta *)meta;

    if (GST_META_TRANSFORM_IS_COPY(type))
    {
        dmeta = (_ExampleCMeta *)gst_buffer_add_meta(dest,
                                                     EXAMPLE_C_META_INFO, NULL);

        if (!dmeta)
            return FALSE;

        dmeta->count = smeta->count;
        dmeta->num = smeta->num;
    }
    else
    {
        /* return FALSE, if transform type is not supported */
        return FALSE;
    }
    return TRUE;
}

/*
利用者向けに公開するメタデータ取り出し関数
*/
const GstMetaInfo *
example_c_meta_get_info(void)
{
    static const GstMetaInfo *meta_info = NULL;

    if (g_once_init_enter(&meta_info))
    {
        const GstMetaInfo *meta =
            gst_meta_register(
                EXAMPLE_C_META_API_TYPE,
                "ExampleCMeta",
                sizeof(ExampleCMeta),
                gst_example_c_meta_init,
                gst_example_c_meta_free,
                gst_example_c_meta_transform);
        g_once_init_leave(&meta_info, meta);
    }
    return meta_info;
}

_ExampleCMeta *
buffer_add_example_c_meta(GstBuffer *buffer, guint64 count, gfloat num)
{
    _ExampleCMeta *meta;

    g_return_val_if_fail(GST_IS_BUFFER(buffer), NULL);

    meta = (_ExampleCMeta *)gst_buffer_add_meta(buffer,
                                                EXAMPLE_C_META_INFO, NULL);

    meta->count = count;
    meta->num = num;
    return meta;
}
