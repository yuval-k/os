CROSS_TOOL_TARGET ?= arm-none-eabi
ARCH=arm

# BOARD?=rpi2
BOARD?=integrator

ifeq ($(BOARD),rpi2)
TARGET ?= armv7-unknown-linux-gnueabihf
MACHINE=raspi2,firmware=target/kernel.img -smp 4
QEMU=qemu-system-arm
RUSTCFLAGS=
endif

ifeq ($(BOARD),rpi)
# TODO later change to something with floats
TARGET ?= arm-unknown-linux-gnueabi
RUSTCFLAGS=-Ctarget-cpu=arm1176jz-s
endif

ifeq ($(BOARD),integrator)
TARGET ?= arm-unknown-linux-gnueabi
MACHINE=integratorcp -m 128 -cpu arm1176
QEMU=qemu-system-arm
RUSTCFLAGS=-Ctarget-cpu=arm1176jz-s
endif


linker_script=src/arch/$(ARCH)/board/$(BOARD)/linker.ld
stub=src/arch/$(ARCH)/board/$(BOARD)/stub.S 
stub_object=target/$(TARGET)/stub.o
os_lib=target/$(TARGET)/debug/libos.a

glue=src/arch/$(ARCH)/board/$(BOARD)/glue.c
glue_object=target/$(TARGET)/glue.o

LIB_COMPILER=$(shell find  ~/.multirust/ -name $(TARGET))/lib/libcompiler_builtins*.rlib


AS=$(TARGET)-as
CPP=$(TARGET)-cpp
LD=$(TARGET)-ld
CC=$(TARGET)-cc
OBJCOPY=$(TARGET)-objcopy

.PHONY: toolchain
toolchain:
	rustup override add nightly
	rustup target add $(TARGET)
	
emulate: target/kernel.elf
	$(QEMU) -machine $(MACHINE) -kernel target/kernel.elf -serial stdio -s

emulate-debug: target/kernel.raw
	$(QEMU) -machine $(MACHINE) -kernel target/kernel.raw -serial stdio -s -S

.PHONY: cargo
cargo:
	# see here: https://mail.mozilla.org/pipermail/rust-dev/2014-March/009153.html
	cargo rustc --features board-$(BOARD) --target=$(TARGET) --  $(RUSTCFLAGS) 

$(os_lib): cargo

$(stub_object): $(stub)
	$(CPP) $(stub) |  $(AS) -g -o $(stub_object)

$(glue_object): $(glue)
	$(CC) -Wall -Wextra -Werror -nostdlib -nostartfiles -ffreestanding -std=gnu99 -c $(glue) -o $(glue_object)

target/kernel.elf: $(os_lib) $(linker_script) $(stub_object) $(glue_object)
	$(LD) -n --gc-sections -T $(linker_script) -o target/kernel.elf \
		$(stub_object) $(glue_object) $(os_lib) $(LIB_COMPILER)

target/kernel.raw: target/kernel.elf
	$(OBJCOPY) -S -O binary target/kernel.elf target/kernel.raw

target/kernel7.img: target/kernel.raw
	cp target/kernel.raw target/kernel7.img

target/kernel.img: target/kernel.raw
	cp target/kernel.raw target/kernel.img

build: target/kernel.elf target/kernel.raw target/kernel.img target/kernel7.img

debugosx: build container
	# qemu-system-arm -machine versatilepb -cpu arm1136 -m 128 -kernel target/kernel.elf -s -S&
	@echo Now use this command to debug:
	@echo docker run  --rm -t -i -v $(shell pwd):$(shell pwd):ro  -v $(shell cd;pwd)/.cargo/registry/src/:$(shell cd;pwd)/.cargo/registry/src/:ro --net="host" arm-cross-tools
	@echo Followed by:
	@echo arm-none-eabi-gdb -ex \'target remote 192.168.99.1:1234\' $(shell pwd)/target/kernel.elf

.PHONY: clean
clean:
	cargo clean

.PHONY: container
container:
	docker build -t arm-cross-tools tools/arm-cross-tools
	docker build -t qemu-rpi tools/qemu
# cargo:
# 	cargo build --target $(TARGET)
#	cargo rustc --target $(TARGET) -- -Z no-landing-pads
