# Path and Variables
SHELL := /bin/bash
PROJECT := paper-scissors-stone
ROOT_DIR := $(CURDIR)

###########################################################
### Local Deployment
build:
	source ./environment.sh ;\
	cargo +nightly build -Z build-std=std,core,alloc,panic_abort,proc_macro --target riscv64ima-cartesi-linux-gnu.json --release

###########################################################
### Docker

# docker.io/cartesi/rootfs:devel image is built from https://github.com/cartesi/image-rootfs
build-env:
	docker run -it --rm -h playground \
		-u root \
		-v ${ROOT_DIR}:/app \
		-v ${ROOT_DIR}/target:/app/target \
		-w /app \
		docker.io/cartesi/rootfs:devel /bin/bash

build-linux:
	docker run --rm \
	-u root \
	-v ${ROOT_DIR}:/app \
	-v ${ROOT_DIR}/target:/app/target \
	-w /app \
	docker.io/cartesi/rootfs:devel /bin/bash -c "make build"
