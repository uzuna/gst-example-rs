

.PHONY: build
build: target/debug/libgstrsexample.so plugin/src
	cargo build

.PHONY: inspect
inspect: build
	gst-inspect-1.0 --gst-plugin-path=target/debug target/debug/libgstrsexample.so

.PHONY: run.trans
run.trans: build
	GST_DEBUG=1,testtrans:7 gst-launch-1.0 --gst-plugin-path=target/debug videotestsrc ! testtrans ! autovideosink

.PHONY: fmt
fmt:
	cargo fmt
	cargo clippy --fix --allow-staged

.PHONY: check-fmt
check-fmt:
	cargo fmt --check
	cargo clippy