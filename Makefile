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
	libluxtorpeda.so \
	luxtorpeda.pck \
	luxtorpeda.sh \
	luxtorpeda.x86_64 \
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

GODOT := ~/.local/share/Steam/steamapps/common/Godot\ Engine/godot.x11.opt.tools.64
SCONS := scons

install_dir = $(DESTDIR)/$(PREFIX)/share/steam/compatibilitytools.d/$(tool_dir)

dev_install_dir = $(data_home)/Steam/compatibilitytools.d/$(tool_dir_dev)

build:
	cargo build
	$(GODOT) --path . --export "Linux/X11" target/debug/luxtorpeda.x86_64 --no-window

release:
	cargo build --release
	mkdir -p target/debug
	cp -r target/release/* target/debug
	$(GODOT) --path . --export "Linux/X11" target/release/luxtorpeda.x86_64 --no-window

lint:
	cargo clippy -- -D warnings

test:
	cargo test

clean:
	cargo clean
	rm -rf $(tool_dir)
	rm -f $(tool_dir).tar.xz
	rm -rf godot-build

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
	$(STRIP) luxtorpeda/libluxtorpeda.so

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

godot-export-template:
	# scons is needed for this
	rm -rf godot-build
	git clone https://github.com/godotengine/godot.git --depth 1 -b 3.5.1-stable godot-build
	cd godot-build && $(SCONS) platform=x11 tools=no target=release bits=64 lto=full disable_3d=yes module_arkit_enabled=no module_assimp_enabled=no module_bullet_enabled=no module_camera_enabled=no module_csg_enabled=no module_dds_enabled=no module_enet_enabled=no module_etc_enabled=no module_gridmap_enabled=no module_hdr_enabled=no module_jsonrpc_enabled=no module_mbedtls_enabled=no module_mobile_vr_enabled=no module_opensimplex_enabled=no module_opus_enabled=no module_pvr_enabled=no module_recast_enabled=no module_regex_enabled=no module_tga_enabled=no module_theora_enabled=no module_tinyexr_enabled=no module_upnp_enabled=no module_vhacd_enabled=no module_vorbis_enabled=no module_webm_enabled=no module_webrtc_enabled=no module_websocket_enabled=no module_xatlas_unwrap_enabled=no module_svg_enabled=no optimize=size minizip=no module_freetype_enabled=no module_gdnavigation_enabled=no module_ogg_enabled=no module_minimp3_enabled=no module_stb_vorbis_enabled=no module_visual_script_enabled=no module_webp_enabled=no && strip ./bin/godot.x11.opt.64
