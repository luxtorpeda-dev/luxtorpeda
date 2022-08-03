.PHONY: all build test clean doc install user-install user-uninstall

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
	luxtorpeda \
	luxtorpeda.sh \
	config.json \
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
	cp -r --reflink=auto $< $@

target/release/%: %
	cp -r --reflink=auto $< $@

$(tool_dir): \
		release \
		target/release/compatibilitytool.vdf \
		target/release/toolmanifest.vdf \
		target/release/config.json \
		target/release/luxtorpeda.sh \
		target/release/LICENSE \
		target/release/README.md
	mkdir -p $(tool_dir)
	cd target/release && cp -r --reflink=auto -t ../../$(tool_dir) $(files)
	$(STRIP) luxtorpeda/luxtorpeda

$(tool_dir).tar.xz: $(tool_dir)
	@if [ "$(version)" != "" ]; then\
		echo "$(version)" > "$(tool_dir)/version";\
	fi

	tar -cJf $@ $(tool_dir)

install: $(tool_dir)
	mkdir -p $(install_dir)
	cp -av $(tool_dir)/* $(install_dir)/

user-install: \
		build \
		target/debug/compatibilitytool.vdf \
		target/debug/toolmanifest.vdf \
		target/debug/config.json \
		target/debug/luxtorpeda.sh \
		target/debug/LICENSE \
		target/debug/README.md
	mkdir -p $(dev_install_dir)
	cd target/debug && cp -r --reflink=auto -t $(dev_install_dir) $(files)

user-uninstall:
	rm -rf $(dev_install_dir)

check-formatting:
	cargo fmt -- --check
