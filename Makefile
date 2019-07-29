.PHONY: build test clean doc user-install user-uninstall

# These variables are used to generate compatibilitytool.vdf:
#
tool_name             = luxtorpeda
tool_name_dev         = luxtorpeda_dev
tool_name_display     = Luxtorpeda
tool_name_display_dev = Luxtorpeda (dev)

# Default names for installation directories:
#
tool_dir     = luxtorpeda
tool_dir_dev = luxtorpeda-dev

files = compatibilitytool.vdf \
	toolmanifest.vdf \
	packages.json \
	luxtorpeda \
	LICENSE \
	README.md

ifeq ($(origin XDG_DATA_HOME), undefined)
	data_home := ${HOME}/.local/share
else
	data_home := ${XDG_DATA_HOME}
endif

STRIP := strip

PREFIX := /usr/local

install_dir = $(DESTDIR)/$(PREFIX)/share/steam/compatibilitytools.d/$(tool_dir)

dev_install_dir = $(data_home)/Steam/compatibilitytools.d/$(tool_dir_dev)

.PHONY: all

all: release

build:
	cargo build

release:
	cargo build --release

lint:
	cargo clippy -- -D warnings

test:
	cargo test

clean:
	cargo clean
	rm -rf $(tool_dir)
	rm -f $(tool_dir).tar.xz

doc:
	cargo doc --document-private-items --open

target/debug/compatibilitytool.vdf: compatibilitytool.template
	sed 's/%name%/$(tool_name_dev)/; s/%display_name%/$(tool_name_display_dev)/' $< > $@

target/release/compatibilitytool.vdf: compatibilitytool.template
	sed 's/%name%/$(tool_name)/; s/%display_name%/$(tool_name_display)/' $< > $@

target/debug/%: %
	cp --reflink=auto $< $@

target/release/%: %
	cp --reflink=auto $< $@

$(tool_dir): \
		release \
		target/release/compatibilitytool.vdf \
		target/release/toolmanifest.vdf \
		target/release/packages.json \
		target/release/LICENSE \
		target/release/README.md
	mkdir -p $(tool_dir)
	cd target/release && cp --reflink=auto -t ../../$(tool_dir) $(files)

$(tool_dir).tar.xz: $(tool_dir)
	$(STRIP) luxtorpeda/luxtorpeda
	tar -cJf $@ $(tool_dir)

install: $(tool_dir)
	mkdir -p $(install_dir)
	cp -av $(tool_dir)/* $(install_dir)/

user-install: \
		build \
		target/debug/compatibilitytool.vdf \
		target/debug/toolmanifest.vdf \
		target/debug/packages.json \
		target/debug/LICENSE \
		target/debug/README.md
	mkdir -p $(dev_install_dir)
	cd target/debug && cp --reflink=auto -t $(dev_install_dir) $(files)

user-uninstall:
	rm -rf $(dev_install_dir)

check-formatting:
	cargo fmt -- --check
