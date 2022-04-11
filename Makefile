# Path and Variables
SHELL := /bin/bash
PROJECT := paper-scissors-stone
ROOT_DIR := $(CURDIR)

###########################################################
### Local Deployment
build:
	source ~/.cargo/env ;\
	source ./environment.sh ;\
	cargo +nightly build -Z build-std=std,core,alloc,panic_abort,proc_macro --target riscv64ima-cartesi-linux-gnu.json --release

###########################################################
### Docker
build-linux:
	docker run --rm \
	-u root \
	-v ${ROOT_DIR}:/app \
	-v ${ROOT_DIR}/target/x86_64-unknown-linux-gnu:/app/target \
	-w /app \
	maxaetheras/cartesi-rust-build-env:latest /bin/bash -c "make build"
