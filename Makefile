TARGET ?= arm-unknown-linux-gnueabi
ARCH=arm
BOARD=integrator

linker_script=src/arch/$(ARCH)/$(BOARD)/linker.ld
stub=src/arch/$(ARCH)/$(BOARD)/stub.S 
stub_object=target/$(TARGET)/stub.o
os_lib=target/$(TARGET)/debug/libos.a

AS=$(TARGET)-as
CPP=$(TARGET)-cpp
LD=$(TARGET)-ld

.PHONY: toolchain
toolchain:
	rustup override add nightly
	rustup target add $(TARGET)
	
emulate: target/kernel.img
	qemu-system-arm -machine integratorcp -m 128 -kernel target/kernel.img -serial stdio

$(os_lib):
	cargo build --target=$(TARGET)

$(stub_object): $(stub)
	$(CPP) $(stub) |  $(AS)  -o $(stub_object)

target/kernel.img: $(os_lib) $(linker_script) $(stub_object)
	$(LD) -n --gc-sections -T $(linker_script) -o target/kernel.img \
		$(stub_object) target/$(TARGET)/debug/libos.a

build: target/kernel.img

# cargo:
# 	cargo build --target $(TARGET)
#	cargo rustc --target $(TARGET) -- -Z no-landing-pads