docker-build:
	docker build -t lol:perf - < Dockerfile

.PHONY: binary
binary:
	cargo build --release

stress:
	cargo run --release -- --runtime=300

flamegraph:
	cargo flamegraph

callgrind:
	cargo profiler callgrind --release -n 20

# BUG
# https://github.com/svenstaro/cargo-profiler/issues/60
cachegrind:
	cargo profiler cachegrind --release -n 20