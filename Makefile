# Path and Variables
SHELL := /bin/bash
PROJECT := paper-scissors-stone
ROOT_DIR := $(CURDIR)
TOOLCHAIN_TAG := 0.9.0
TOOLCHAIN_IMG := cartesi/toolchain:$(TOOLCHAIN_TAG)

###########################################################
### Local Deployment
debug:
	cargo run mono

build:
	source ./environment.sh ;\
	cargo +nightly build -Z build-std=std,core,alloc,panic_abort,proc_macro --target riscv64ima-cartesi-linux-gnu.json --release

###########################################################
### Docker

build-env:
	docker run -it --rm -h playground \
		-u root \
		-v ${ROOT_DIR}:/app \
		-v ${ROOT_DIR}/target:/app/target \
		-w /app \
		${TOOLCHAIN_IMG} /bin/bash

build-linux:
	docker run --rm \
	-u root \
	-v ${ROOT_DIR}:/app \
	-v ${ROOT_DIR}/target:/app/target \
	-w /app \
	${TOOLCHAIN_IMG} /bin/bash -c "make build"
