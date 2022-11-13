include ../../common.mk

SO_NAME:=libexample_c_meta.so
RUST_TEST_DEPS_OUT_DIR:=${OUT_DIR}/deps

CC := g++

GST_INSTALL_DIR:=
LIB_INSTALL_DIR:=
CFLAGS:=
LIBS:= -lpthread

SRCS := $(wildcard *.cpp)
INCLUDES += -I./

PKGS := gstreamer-1.0 \
	gstreamer-base-1.0 \
	gstreamer-video-1.0 \
	gstreamer-allocators-1.0 \
	glib-2.0


OBJS := $(SRCS:.cpp=.o)

CFLAGS += -fPIC -O3

CFLAGS += `pkg-config --cflags $(PKGS)`

LDFLAGS = -Wl,--no-undefined -L$(LIB_INSTALL_DIR) -Wl,-rpath,$(LIB_INSTALL_DIR)

LIBS += `pkg-config --libs $(PKGS)`

all: $(OUT_DIR)/$(SO_NAME)

%.o: %.cpp %.h
	$(CC) -c $< $(CFLAGS) $(INCLUDES) -o $@


$(OUT_DIR)/$(SO_NAME): $(OBJS)
	mkdir -p $(OUT_DIR)
	$(CC) -shared -o $(OUT_DIR)/$(SO_NAME) $(OBJS) $(LIBS) $(LDFLAGS)
	# copy for cargo test
	cp -f $(OUT_DIR)/$(SO_NAME) $(RUST_TEST_DEPS_OUT_DIR)/$(SO_NAME)

.PHONY: clean
clean:
	rm -rf $(OBJS) $(OUT_DIR)/$(SO_NAME)
