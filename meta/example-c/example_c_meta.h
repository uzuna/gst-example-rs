#ifndef __GST_EXAMPLE_C_META_H__
#define __GST_EXAMPLE_C_META_H__

#include <gst/gst.h>

G_BEGIN_DECLS

typedef struct _ExampleCMeta ExampleCMeta;
typedef struct _ExampleCMetaParam ExampleCMetaParam;

struct _ExampleCMeta {
    GstMeta meta;

    gint64 count;
    gfloat num;
};

struct _ExampleCMetaParam {
    gint64 count;
    gfloat num;
};

// api get type
GType example_c_meta_api_get_type (void);
#define EXAMPLE_C_META_API_TYPE (example_c_meta_api_get_type())

// get info
const GstMetaInfo * example_c_meta_get_info (void);
#define EXAMPLE_C_META_INFO (example_c_meta_get_info())

// utility function
ExampleCMeta * buffer_add_example_c_meta (GstBuffer * buffer, gint64 count, gfloat num);

// utility function
ExampleCMeta * buffer_add_param_example_c_meta (GstBuffer * buffer, ExampleCMetaParam *param);

G_END_DECLS

#endif /* __GST_EXAMPLE_META_H__ */
