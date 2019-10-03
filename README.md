# Luxtorpeda

[![Luxtorpeda project Discord](https://img.shields.io/discord/514567252864008206.svg?label=discord)](https://discord.gg/8mFhUPX)
[![Say Thanks!](https://img.shields.io/badge/Say%20Thanks-!-1EAEDB.svg)](https://saythanks.io/to/dreamer)

Steam Play compatibility tool to run games using native Linux engines; [project overview](https://github.com/dreamer/luxtorpeda/wiki).

This is a sister project of
[Boxtron](https://github.com/dreamer/boxtron/) and
[Roberta](https://github.com/dreamer/roberta).

![screenshot-0](https://user-images.githubusercontent.com/3967/61964568-7b674500-afce-11e9-9c42-ef6cc1b425b6.png)

Official mirrors:
[GitHub](https://github.com/dreamer/luxtorpeda),
[GitLab](https://gitlab.com/luxtorpeda/luxtorpeda).

## Pre-requisites

Compatibility tool will work on any modern 64-bit Linux distribution.
All packaged games are compiled for Steam Runtime environment and require no
additional dependencies.

**Using Luxtorpeda with [Steam native runtime](https://wiki.archlinux.org/index.php/Steam/Troubleshooting#Steam_native_runtime) may or may not work, but is not supported.**

## Installation (using tarball)

*This is pre-release quality software, expect bugs and missing features.*

1. Close Steam.
2. Download and unpack tarball to compatibilitytools.d directory (create one if it does not exist):

        $ cd ~/.local/share/Steam/compatibilitytools.d/ || cd ~/.steam/root/compatibilitytools.d/
        $ curl -L https://luxtorpeda.gitlab.io/luxtorpeda/master/luxtorpeda.tar.xz | tar xJf -

3. Start Steam.
4. In game properties window select "Force the use of a specific Steam Play
   compatibility tool" and select "Luxtorpeda".

## Installation (debug build, from source)

0. Download the latest version of Rust: https://www.rust-lang.org/ and verify that openssl is installed on your system.

Debian, Ubuntu et consortes

       $ sudo apt install libssl-dev
       
Fedora 

       $ sudo dnf install openssl-devel
       
1. Close Steam.
2. Clone the repository, then use makefile to trigger `cargo build` and install:

       $ git clone https://github.com/dreamer/luxtorpeda.git
       $ cd luxtorpeda
       $ make user-install

3. Start Steam.
4. In game properties window select "Force the use of a specific Steam Play
   compatibility tool" and select "Luxtorpeda&nbsp;(dev)".


## Known issues

- If game does not start with error *"No command line defined"*, it means you
need to force-update game package to a new format.  Find game package tarball
in `~/.cache/luxtorpeda/<appid>/`, remove it, and start the game again.

- Solus and Mint users might be affected by unintended dependency on `libssl.so.1.1`,
see [#42](https://github.com/dreamer/luxtorpeda/issues/42) for details.


## Supported titles

Just click "Play" and Luxtorpeda will download and install the package for you.
You need to select Luxtorpeda as a compatibility tool first, of course.

| Game                                                                              | Engine                                                      | Engine&nbsp;Version | Comments
|---                                                                                |---                                                          |---                  |---
| [The Elder Scrolls III: Morrowind](https://store.steampowered.com/app/22320/)     | [OpenMW](https://openmw.org/)                               | `0.45.0`            | *Missing GUI mod launcher for now: [#17](https://github.com/dreamer/luxtorpeda/issues/17)*
| [Arx Fatalis](https://store.steampowered.com/app/1700/)                           | [Arx&nbsp;Libertatis](https://arx-libertatis.org/)          | `db77aa`            | *Textures issue: [#27](https://github.com/dreamer/luxtorpeda/issues/27)*
| [Arx Fatalis Demo](https://steamdb.info/app/1710/)                                | [Arx&nbsp;Libertatis](https://arx-libertatis.org/)          | `db77aa`            | **(Free to play)** *Textures issue: [#27](https://github.com/dreamer/luxtorpeda/issues/27)*
| [Quake](https://store.steampowered.com/app/2310/)                                 | [vkQuake](https://github.com/Novum/vkQuake)                 | `1.02.1`            | *Single-player only for now, fixed music*
| [Quake II](https://store.steampowered.com/app/2320/)                              | [Yamagi&nbsp;Quake&nbsp;II](https://www.yamagi.org/quake2/) | `7.41`              | *Fixed default config, fixed music*
| [Quake II Demo](https://steamdb.info/app/9130/)                                   | [Yamagi&nbsp;Quake&nbsp;II](https://www.yamagi.org/quake2/) | `7.41`              | **(Free to play)** *Fixed default config*
| [Quake II: The Reckoning](https://store.steampowered.com/app/2330/)               | [Yamagi&nbsp;Quake&nbsp;II](https://www.yamagi.org/quake2/) | `7.41`              | *Fixed default config, fixed music*
| [Quake II: Ground Zero](https://store.steampowered.com/app/2340/)                 | [Yamagi&nbsp;Quake&nbsp;II](https://www.yamagi.org/quake2/) | `7.41`              | *Fixed default config, fixed music*
| [Quake III Arena](https://store.steampowered.com/app/2200/)                       | [ioquake3](https://ioquake3.org/)                           | `a96f32`            |
| [Quake III: Team Arena](https://store.steampowered.com/app/2350/)                 | [ioquake3](https://ioquake3.org/)                           | `a96f32`            |
| [Jedi Knight II: Jedi Outcast](https://store.steampowered.com/app/6030/)          | [OpenJK](https://github.com/JACoders/OpenJK)                | `e8b5c1`            | *Single-player only for now*
| [Jedi Knight: Jedi Academy (SP)](https://store.steampowered.com/app/6020/)        | [OpenJK](https://github.com/JACoders/OpenJK)                | `e8b5c1`            | *Single-player*
| [Jedi Knight: Jedi Academy (MP)](https://store.steampowered.com/app/6020/)        | [EternalJK](https://github.com/eternalcodes/EternalJK)      | `1.5.6-1`           | *Multi-player*
| [X-COM: UFO Defense](https://store.steampowered.com/app/7760/)                    | [OpenXcom](https://openxcom.org/)                           | `1.0-2835`          |
| [X-COM: Terror from the Deep](https://store.steampowered.com/app/7650/)           | [OpenXcom](https://openxcom.org/)                           | `1.0-2835`          |
| [Return to Castle Wolfenstein](https://store.steampowered.com/app/9010/)          | [iortcw](https://github.com/iortcw/iortcw)                  | `1.51c`             | *Both SP and MP*
| [Doom (1993)](https://store.steampowered.com/app/2280/)                           | [GZDoom](https://zdoom.org/)                                | `4.2.1`             | *"The Ultimate DOOM"; Vulkan renderer crashes on exit*
| [Doom II: Hell on Earth](https://store.steampowered.com/app/2300/)                | [GZDoom](https://zdoom.org/)                                | `4.2.1`             | *Vulkan renderer crashes on exit*
| [Final Doom](https://store.steampowered.com/app/2290/)                            | [GZDoom](https://zdoom.org/)                                | `4.2.1`             | *Vulkan renderer crashes on exit*
| [Doom 3](https://store.steampowered.com/app/9050/)                                | [dhewm3](https://dhewm3.org/)                               | `1.5.1-PRE1`        |
| [Doom 3: Resurrection of Evil](https://store.steampowered.com/app/9070/)          | [dhewm3](https://dhewm3.org/)                               | `1.5.1-PRE1`        |
| [Heretic: Shadow of the Serpent Riders](https://store.steampowered.com/app/2390/) | [GZDoom](https://zdoom.org/)                                | `4.2.1`             | *Vulkan renderer crashes on exit*
| [Hexen: Beyond Heretic](https://store.steampowered.com/app/2360/)                 | [GZDoom](https://zdoom.org/)                                | `4.2.1`             | *Vulkan renderer crashes on exit*
| [Doki Doki Literature Club!](https://store.steampowered.com/app/698780/)          | [Ren'Py](https://www.renpy.org/)                            |                     | **(Free to play)** *Using Linux version bundled with Windows version*

Want a specific game? Maybe we are
[already working on it](https://github.com/dreamer/luxtorpeda/wiki/Game-engines#on-agenda-wip-and-supported-engines).
You can also
[make a package request](https://github.com/dreamer/luxtorpeda/issues/new),
or
[create package yourself](https://github.com/dreamer/luxtorpeda/wiki/Packaging-tutorial)! :)

## Development

You can use `cargo` as with any Rust project; `make` serves only as a convenient
frontend for packaging and triggering longer `cargo` commands.

TODO: Add documentation about packaging games for Luxtorpeda.
