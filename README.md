# Luxtorpeda

[![luxtorpeda discord](https://img.shields.io/discord/514567252864008206.svg?label=discord)](https://discord.gg/8mFhUPX)

Compatibility tool to run games on Steam using native Linux engines; [project overview](https://github.com/dreamer/luxtorpeda/wiki).

This is a sister project of [Boxtron](https://github.com/dreamer/boxtron/).

![screenshot-0](https://user-images.githubusercontent.com/3967/61964568-7b674500-afce-11e9-9c42-ef6cc1b425b6.png)

Official mirrors:
[GitHub](https://github.com/dreamer/luxtorpeda),
[GitLab](https://gitlab.com/luxtorpeda/luxtorpeda).

## Pre-requisites

Compatibility tool will work on any modern 64-bit Linux distribution.
All packaged games are compiled for Steam Runtime environment and require no
additional dependencies. **Native Runtime is not supported.**

## Installation (using tarball)

*This is pre-release quality software, expect bugs and missing features*

1. Close Steam.
2. Download and unpack tarball to compatibilitytools.d directory (create one if it does not exist):

        $ cd ~/.local/share/Steam/compatibilitytools.d/ || cd ~/.steam/root/compatibilitytools.d/
        $ curl -L https://luxtorpeda.gitlab.io/luxtorpeda/master/luxtorpeda.tar.xz | tar xJf -

3. Start Steam.
4. In game properties window select "Force the use of a specific Steam Play
   compatibility tool" and select "Luxtorpeda".

## Installation (debug build, from source)

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

Just click "Play" and Luxtorpeda will download and install the package for you.
You need to select Luxtorpeda as a compatibility tool first, of course.

| Game | Engine | Package | Version | Comments
|---   |---     |---	  |---      |---
| [Quake III Arena](https://store.steampowered.com/app/2200/) | [ioquake3](https://ioquake3.org/) | [link](https://luxtorpeda.gitlab.io/packages/ioq3/) | |
| [Jedi Knight: Jedi Academy](https://store.steampowered.com/app/6020/) | [OpenJK](https://github.com/JACoders/OpenJK) | [link](https://luxtorpeda.gitlab.io/packages/openjk/) | | *Single player only for now*
| [Jedi Knight II: Jedi Outcast](https://store.steampowered.com/app/6030/) | [OpenJK](https://github.com/JACoders/OpenJK) | [link](https://luxtorpeda.gitlab.io/packages/openjk/) | | *Single player only; in development*
| [X-COM: UFO Defense](https://store.steampowered.com/app/7760/) | [OpenXcom](https://openxcom.org/) | [link](https://luxtorpeda.gitlab.io/packages/openxcom/) | |
| [X-COM: Terror from the Deep](https://store.steampowered.com/app/7650/) | [OpenXcom](https://openxcom.org/) | [link](https://luxtorpeda.gitlab.io/packages/openxcom/) | |
| [Return to Castle Wolfenstein](https://store.steampowered.com/app/9010/) | [iortcw](https://github.com/iortcw/iortcw) | [link](https://luxtorpeda.gitlab.io/packages/iortcw/) | | *Both SP and MP*
| [Doom (1993)](https://store.steampowered.com/app/2280/) | [GZDoom](https://zdoom.org/) | [link](https://luxtorpeda.gitlab.io/packages/gzdoom/) | 4.1.3-1 | *"The Ultimate DOOM"; Vulkan renderer disabled*
| [Doom II: Hell on Earth](https://store.steampowered.com/app/2300/) | [GZDoom](https://zdoom.org/) | [link](https://luxtorpeda.gitlab.io/packages/gzdoom/) | 4.1.3-1 | *Vulkan renderer disabled*
| [Final Doom](https://store.steampowered.com/app/2290/) | [GZDoom](https://zdoom.org/) | [link](https://luxtorpeda.gitlab.io/packages/gzdoom/) | 4.1.3-1 | *Vulkan renderer disabled*
| [Doom 3](https://store.steampowered.com/app/9050/) | [dhewm3](https://dhewm3.org/) | [link](https://luxtorpeda.gitlab.io/packages/dhewm3/) | |
| [Doom 3: Resurrection of Evil](https://store.steampowered.com/app/9070/) | [dhewm3](https://dhewm3.org/) | [link](https://luxtorpeda.gitlab.io/packages/dhewm3/) | |
| [Heretic: Shadow of the Serpent Riders](https://store.steampowered.com/app/2390/) | [GZDoom](https://zdoom.org/) | [link](https://luxtorpeda.gitlab.io/packages/gzdoom/) | 4.1.3-1 | *Vulkan renderer disabled*
| [Hexen: Beyond Heretic](https://store.steampowered.com/app/2360/) | [GZDoom](https://zdoom.org/) | [link](https://luxtorpeda.gitlab.io/packages/gzdoom/) | 4.1.3-1 | *Vulkan renderer disabled*
| [Doki Doki Literature Club!](https://store.steampowered.com/app/698780/) | [Ren'Py](https://www.renpy.org/) | N/A | | *Using Linux version bundled with Windows version*
| [Arx Fatalis](https://store.steampowered.com/app/1700/) | [Arx Libertatis](https://arx-libertatis.org/) | [link](https://luxtorpeda.gitlab.io/packages/arxlibertatis/) | | 

Want more games? [Make a feature request](https://github.com/dreamer/luxtorpeda/issues/new) or [create package yourself](https://github.com/dreamer/luxtorpeda/wiki/Packaging-tutorial)! :)

## Development

You can use `cargo` as with any Rust project; `make` serves only as a convenient
frontend for packaging and triggering longer `cargo` commands.

TODO: Add documentation about packaging games for Luxtorpeda.
