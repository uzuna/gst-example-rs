

.PHONY: build
build: target/debug/libgstrsexample.so
target/debug/libgstrsexample.so:
	cargo build

.PHONY: inspect
inspect:
	gst-inspect-1.0 --gst-plugin-path=target/debug target/debug/libgstrsexample.so
