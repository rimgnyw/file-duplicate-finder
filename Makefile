LINUX_TARGET = x86_64-unknown-linux-gnu
WINDOWS_TARGET = x86_64-pc-windows-gnu

TARGETS = $(LINUX_TARGET) $(WINDOWS_TARGET)

.PHONY: all clean run

all: clean get_targets linux windows

run:
	cd scanner; \
	cargo build

	cd gui; \
	cargo build

	cp scanner/target/debug/file-duplicate-finder dist
	cp gui/target/debug/gui dist

	cd dist; \
	./gui

linux:
	cd scanner; \
	cargo build --release --target $(LINUX_TARGET)

	cd gui; \
	cargo build --release --target $(LINUX_TARGET)

	mkdir -p dist/linux

	cp scanner/target/$(LINUX_TARGET)/release/file-duplicate-finder dist/linux 
	cp   gui/target/$(LINUX_TARGET)/release/gui dist/linux

windows:
	cd scanner; \
	cargo build --release --target $(WINDOWS_TARGET)

	cd gui; \
	cargo build --release --target $(WINDOWS_TARGET)

	mkdir -p dist/windows

	cp scanner/target/$(WINDOWS_TARGET)/release/file-duplicate-finder.exe dist/windows/file-duplicate-finder-win64.exe
	 cp  gui/target/$(WINDOWS_TARGET)/release/gui.exe dist/windows

get_targets:
	$(foreach t,$(TARGETS), \
		rustup target add $(t);)


clean:
	rm -rfv dist/*
