.PHONY: all build clean install user-install user-uninstall

tool_dir     = luxtorpeda
tool_dir_dev = luxtorpeda-dev

ifeq ($(origin XDG_DATA_HOME), undefined)
	data_home := ${HOME}/.local/share
else
	data_home := ${XDG_DATA_HOME}
endif

PREFIX := /usr/local

GODOT := ~/.distrobox/steam/.local/share/Steam/steamapps/common/Godot\ Engine/godot.x11.opt.tools.64

install_dir = $(DESTDIR)/$(PREFIX)/share/steam/compatibilitytools.d/$(tool_dir)

dev_install_dir = $(data_home)/Steam/compatibilitytools.d/$(tool_dir_dev)

build:
	cargo install cargo-post
	GODOT=$(GODOT) TARGET=$(MAKECMDGOALS) cargo post build

release:
	cargo install cargo-post
	GODOT=$(GODOT) TARGET=$(MAKECMDGOALS) VERSION=$(version) cargo post build --release

clean:
	cargo clean
	rm -rf $(tool_dir)
	rm -f $(tool_dir).tar.xz
	rm -rf godot-build

$(tool_dir): \
		release
	echo "Packaging complete"

$(tool_dir).tar.xz: $(tool_dir)
	echo "Archiving complete"

install: $(tool_dir)
	mkdir -p $(install_dir)
	cp -av $(tool_dir)/* $(install_dir)/

user-install: \
		build
	mkdir -p $(dev_install_dir)
	cp -av $(tool_dir)/* $(dev_install_dir)/

user-uninstall:
	rm -rf $(dev_install_dir)
