ARCH ?= x86_64
TARGET ?= $(ARCH)-unknown-uefi
PROJECT ?= stubby
OVMF_FW := OVMF_CODE.fd
OVMF_VARS = OVMF_VARS.fd
BOOT_DIR := BOOT

qemu_args := -nodefaults -vga std -monitor vc:1440x900 -serial stdio -machine q35,accel=kvm:hvf -no-shutdown -no-reboot  -m 256M
qemu_efi := -drive if=pflash,format=raw,readonly=on,file=$(OVMF_FW)
qemu_efi_vars :=  -drive if=pflash,format=raw,file=$(OVMF_VARS)
qemu_drive := -drive format=raw,file=fat:rw:$(BOOT_DIR)

target_debug := target/$(TARGET)/debug/$(PROJECT).efi
target_release := target/$(TARGET)/release/$(PROJECT).efi

.PHONY: all release debug clean launch-debug launch configure

all: $(target_debug) $(target_release)
debug: $(target_debug)
release: $(target_release)

configure:
	mkdir $(BOOT_DIR)

clean:
	rm -rv $(BOOT_DIR)
	@RUST_TARGET_PATH=$(shell pwd) cargo clean --target $(TARGET)

launch-debug: $(target_debug)
	@RUST_TARGET_PATH=$(shell pwd) cargo +nightly build -Z build-std --target $(TARGET) --verbose
	mkdir -p $(BOOT_DIR)/EFI/BOOT/
	cp -v $(target_debug) $(BOOT_DIR)/EFI/BOOT/BOOTX64.EFI
	@qemu-system-$(ARCH) $(qemu_args) $(qemu_efi) $(qemu_efi_vars) $(qemu_drive)

launch: $(target_release)
	@RUST_TARGET_PATH=$(shell pwd) cargo +nightly build -Z build-std --target $(TARGET) --release
	mkdir -p $(BOOT_DIR)/EFI/BOOT/
	cp -v $(target_release) $(BOOT_DIR)/EFI/BOOT/BOOTX64.EFI
	@qemu-system-$(ARCH) $(qemu_args) $(qemu_efi) $(qemu_efi_vars) $(qemu_drive)

$(target_debug):
	@RUST_TARGET_PATH=$(shell pwd) cargo +nightly build -Z build-std --target $(TARGET) --verbose
	mkdir -p $(BOOT_DIR)/EFI/BOOT/
	cp -v $(target_debug) $(BOOT_DIR)/EFI/BOOT/BOOTX64.EFI

$(target_release):
	@RUST_TARGET_PATH=$(shell pwd) cargo +nightly build -Z build-std --target $(TARGET) --release
	mkdir -p $(BOOT_DIR)/EFI/BOOT/
	cp -v $(target_release) $(BOOT_DIR)/EFI/BOOT/BOOTX64.EFI
