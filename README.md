# Luxtorpeda

[![luxtorpeda discord](https://img.shields.io/discord/514567252864008206.svg?label=discord)](https://discord.gg/8mFhUPX)

Compatibility tool to run games on Steam using native Linux engines; [project overview](https://github.com/dreamer/luxtorpeda/wiki).

This is a sister project of [steam-dos](https://github.com/dreamer/steam-dos/).

Official mirrors:
[GitHub](https://github.com/dreamer/luxtorpeda),
[GitLab](https://gitlab.com/luxtorpeda/luxtorpeda).

## Installation (using tarball)

TBD

## Installation (from source)

0. Download the latest version of Rust: https://www.rust-lang.org/
1. Close Steam.
2. Clone the repository, then use makefile to trigger `cargo build` and install:

       $ git clone https://github.com/dreamer/luxtorpeda.git
       $ cd luxtorpeda
       $ make user-install

3. Start Steam.
4. In game properties window select "Force the use of a specific Steam Play
   compatibility tool" and select "Luxtorpeda&nbsp;(dev)".

## Supported titles

| Game | Engine | Package | Comments
|:---	|---	  |---	     |---
| [Quake III Arena](https://store.steampowered.com/app/2200/) | [ioquake3](https://ioquake3.org/) | [link](https://luxtorpeda.gitlab.io/packages/ioq3/) |
| [Jedi Knight: Jedi Academy](https://store.steampowered.com/app/6020/) | [OpenJK](https://github.com/JACoders/OpenJK) | [link](https://luxtorpeda.gitlab.io/packages/openjk/) | *Single player only for now*
| [Jedi Knight II: Jedi Outcast](https://store.steampowered.com/app/6030/) | [OpenJK](https://github.com/JACoders/OpenJK) | [link](https://luxtorpeda.gitlab.io/packages/openjk/) | *Single player only; in development*
| [X-COM: UFO Defense](https://store.steampowered.com/app/7760/) | [OpenXcom](https://openxcom.org/) | [link](https://luxtorpeda.gitlab.io/packages/openxcom/) |
| [X-COM: Terror from the Deep](https://store.steampowered.com/app/7650/) | [OpenXcom](https://openxcom.org/) | [link](https://luxtorpeda.gitlab.io/packages/openxcom/) |
| [Doki Doki Literature Club!](https://store.steampowered.com/app/698780/) | [Ren'Py](https://www.renpy.org/) | N/A | *Using Linux version bundled with Windows version*

Want more games? [Make a feature request](https://github.com/dreamer/luxtorpeda/issues/new) or [create package yourself](https://github.com/dreamer/luxtorpeda/wiki/Packaging-tutorial)! :)

## Development

You can use `cargo` as with any Rust project; `make` serves only as a convenient
frontend for packaging and triggering longer `cargo` commands.

TODO: Add documentation about packaging games for Luxtorpeda.
