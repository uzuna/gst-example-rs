include ../../common.mk

SO_NAME:=libexample_c_meta.so
RUST_TEST_DEPS_OUT_DIR:=$(RUST_OUT_DIR)/deps

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

all: $(RUST_OUT_DIR)/$(SO_NAME) $(RUST_OUT_DIR)

%.o: %.cpp %.h
	$(CC) -c $< $(CFLAGS) $(INCLUDES) -o $@


$(RUST_OUT_DIR)/$(SO_NAME): $(OBJS)
	mkdir -p $(RUST_OUT_DIR)
	$(CC) -shared -o $(RUST_OUT_DIR)/$(SO_NAME) $(OBJS) $(LIBS) $(LDFLAGS)
	# copy for cargo test
	cp -f $(RUST_OUT_DIR)/$(SO_NAME) $(RUST_TEST_DEPS_OUT_DIR)/$(SO_NAME)

.PHONY: clean
clean:
	rm -rf $(OBJS) $(RUST_OUT_DIR)/$(SO_NAME)
