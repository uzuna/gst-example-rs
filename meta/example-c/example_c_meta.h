#ifndef __GST_EXAMPLE_C_META_H__
#define __GST_EXAMPLE_C_META_H__

#include <gst/gst.h>

G_BEGIN_DECLS

typedef struct _ExampleCMeta ExampleCMeta;

struct _ExampleCMeta {
    GstMeta meta;

    guint64 count;
    gfloat num;
};

// api get type
GType example_c_meta_api_get_type (void);
#define EXAMPLE_C_META_API_TYPE (example_c_meta_api_get_type())

// get info
const GstMetaInfo * example_c_meta_get_info (void);
#define EXAMPLE_C_META_INFO (example_c_meta_get_info())

// utility function
ExampleCMeta * gst_buffer_add_example_c_meta (GstBuffer * buffer, guint64 count, gfloat num);

G_END_DECLS

#endif /* __GST_EXAMPLE_META_H__ */
