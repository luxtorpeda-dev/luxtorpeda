.PHONY: all build clean install user-install user-uninstall

tool_dir     = luxtorpeda
tool_dir_dev = luxtorpeda-dev

ifeq ($(origin XDG_DATA_HOME), undefined)
	data_home := ${HOME}/.local/share
else
	data_home := ${XDG_DATA_HOME}
endif

PREFIX := /usr/local

GODOT := ~/.local/share/Steam/steamapps/common/Godot\ Engine/godot.x11.opt.tools.64

install_dir = $(DESTDIR)/$(PREFIX)/share/steam/compatibilitytools.d/$(tool_dir)

dev_install_dir = $(data_home)/Steam/compatibilitytools.d/$(tool_dir_dev)

build:
	GODOT=$(GODOT) TARGET=$(MAKECMDGOALS) cargo post build

release:
	GODOT=$(GODOT) TARGET=$(MAKECMDGOALS) cargo post build --release

clean:
	cargo clean
	rm -rf $(tool_dir)
	rm -f $(tool_dir).tar.xz
	rm -rf godot-build

$(tool_dir): \
		release \
		target/release/compatibilitytool.vdf \
		target/release/toolmanifest.vdf \
		target/release/luxtorpeda.sh \
		target/release/LICENSE \
		target/release/README.md
	mkdir -p $(tool_dir)
	cd target/release && cp -r --reflink=auto -t ../../$(tool_dir) $(files)

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
		target/debug/luxtorpeda.sh \
		target/debug/LICENSE \
		target/debug/README.md
	mkdir -p $(dev_install_dir)
	cd target/debug && cp -r --reflink=auto -t $(dev_install_dir) $(files)

user-uninstall:
	rm -rf $(dev_install_dir)
