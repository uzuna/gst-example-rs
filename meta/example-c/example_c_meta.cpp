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
gpointer paramsはgst_buffer_add_metaで渡されるポインタ
*/
gboolean
gst_example_c_meta_init(GstMeta *meta, gpointer params, GstBuffer *buffer)
{
    _ExampleCMeta *dmeta = (_ExampleCMeta *)meta;
    _ExampleCMetaParam *dparams = (_ExampleCMetaParam *)params;

    dmeta->label = dparams->label;
    dmeta->count = dparams->count;
    dmeta->num = dparams->num;
    return TRUE;
}

/*
Metaデータ破棄時の処理
Heap領域を使う情報の場合はここで開放をする
*/
void gst_example_c_meta_free(GstMeta *meta, GstBuffer *buffer)
{
    _ExampleCMeta *smeta;
    smeta = (_ExampleCMeta *)meta;

    // FIXME double free or corruption (out)になる
    // 到達前に開放されているが誰が開放しているのかわからない
    if ((&smeta->label)->str) {
        g_free(&smeta->label);
    }
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
        dmeta = (_ExampleCMeta *)buffer_add_example_c_meta(dest,
                                                           smeta->label, smeta->count, smeta->num);
        if (!dmeta)
            return FALSE;
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

// 複数の値を分けて入力したい場合
_ExampleCMeta *
buffer_add_example_c_meta(GstBuffer *buffer, GString label, gint64 count, gfloat num)
{
    _ExampleCMeta *meta;
    _ExampleCMetaParam param = {label, count, num};

    g_return_val_if_fail(GST_IS_BUFFER(buffer), NULL);
    meta = (_ExampleCMeta *)gst_buffer_add_meta(buffer,
                                                EXAMPLE_C_META_INFO, &param);
    return meta;
}

// 構造体で渡したい場合
_ExampleCMeta *
buffer_add_param_example_c_meta(GstBuffer *buffer, _ExampleCMetaParam *param)
{
    _ExampleCMeta *meta;

    // TOOD: gst_example_c_meta_free douple freeの原因が分かったら
    // デバッグプリントを削除する
    // g_print("c sizeof %ld\n", sizeof(_ExampleCMetaParam));
    // g_print("c pointer %p\n", param);
    // g_print("c pointer label %p\n", &param->label);

    g_return_val_if_fail(GST_IS_BUFFER(buffer), NULL);
    meta = (_ExampleCMeta *)gst_buffer_add_meta(buffer,
                                                EXAMPLE_C_META_INFO, param);
    return meta;
}
