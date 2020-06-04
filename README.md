# Luxtorpeda-dev

[![Luxtorpeda project Discord](https://img.shields.io/discord/514567252864008206.svg?label=discord)](https://discord.gg/8mFhUPX)

Steam Play compatibility tool to run games using native Linux engines

![screenshot-0](https://user-images.githubusercontent.com/3967/61964568-7b674500-afce-11e9-9c42-ef6cc1b425b6.png)

Official mirrors:
[GitHub](https://github.com/luxtorpeda-dev/luxtorpeda),
[dreamer GitLab](https://gitlab.com/luxtorpeda/luxtorpeda).

## Pre-requisites

Compatibility tool will work on any modern 64-bit Linux distribution.
All packaged games are compiled for Steam Runtime environment and require no
additional dependencies.

**Using Luxtorpeda with [Steam native runtime](https://wiki.archlinux.org/index.php/Steam/Troubleshooting#Steam_native_runtime) may or may not work, but is not supported.**

## Installation (using tarball)

*This is pre-release quality software, expect bugs and missing features.*

1. Close Steam.
2. Download latest version at https://github.com/luxtorpeda-dev/luxtorpeda/releases
3. Move and unpack tarball to compatibilitytools.d directory (create one if it does not exist):

        $ cd ~/.local/share/Steam/compatibilitytools.d/ || cd ~/.steam/root/compatibilitytools.d/
        $ tar xJf luxtorpeda-2.tar.xz

4. Start Steam.
5. In game properties window select "Force the use of a specific Steam Play
   compatibility tool" and select "Luxtorpeda".

## Installation (debug build, from source)

0. Download the latest version of Rust: https://www.rust-lang.org/ and verify that openssl is installed on your system.

Debian, Ubuntu et consortes

       $ sudo apt install libssl-dev
       
Fedora 

       $ sudo dnf install openssl-devel
       
1. Close Steam.
2. Clone the repository, then use makefile to trigger `cargo build` and install:

       $ git clone https://github.com/luxtorpeda-dev/luxtorpeda.git
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

To see a list of supported titles, go to https://luxtorpeda-dev.github.io/packages.html

Want a specific game? 

Check [issues](https://github.com/luxtorpeda-dev/packages/issues) to see if we are working on it.

You can also make a package request by creating a [new issue](https://github.com/luxtorpeda-dev/packages/issues/new/choose)

You can also [create a package yourself](docs/Creating_a_Package.md)

## Development

You can use `cargo` as with any Rust project; `make` serves only as a convenient
frontend for packaging and triggering longer `cargo` commands.
