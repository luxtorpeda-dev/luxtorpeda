# Luxtorpeda-dev

[![Luxtorpeda project Discord](https://img.shields.io/discord/514567252864008206.svg?label=discord)](https://discord.gg/8mFhUPX)

Steam Play compatibility tool to run games using native Linux engines

![screenshot-0](https://user-images.githubusercontent.com/3967/61964568-7b674500-afce-11e9-9c42-ef6cc1b425b6.png)

## Pre-requisites

Compatibility tool will work on any modern 64-bit Linux distribution.
All packaged games are compiled for Steam Runtime Soldier environment and require no
additional dependencies.

**Using Luxtorpeda with [Steam native runtime](https://wiki.archlinux.org/index.php/Steam/Troubleshooting#Steam_native_runtime) may or may not work, but is not supported.**

## Installation (using tarball)

*This is pre-release quality software, expect bugs and missing features.*

*The packages.json for the supported packages and getting the latest versions will get updated on each launch of luxtorpeda, without any input needed from the user. New releases of the luxtorpeda client will need to be downloaded manually, but a new release of the client is not required when a package is created or updated, unless that package depends on a new feature (which will be noted in the release notes).*

1. Close Steam.
2. Download latest version at https://github.com/luxtorpeda-dev/luxtorpeda/releases
3. Move and unpack tarball to compatibilitytools.d directory (create one if it does not exist):

        $ cd ~/.local/share/Steam/compatibilitytools.d/ || cd ~/.steam/root/compatibilitytools.d/
        $ tar xJf luxtorpeda-51.tar.xz

4. Start Steam.
5. In game properties window select "Force the use of a specific Steam Play
   compatibility tool" and select "Luxtorpeda".
   
## Installation (using GUI)

1. Download ProtonUp-Qt from here: https://davidotek.github.io/protonup-qt/#download
2. Run ProtonUp-Qt and select Steam
3. Click `Add Version`, select Luxtorpeda and press `Install`
4. Restart Steam
5. In Steam game properties window select "Force the use of a specific Steam Play
   compatibility tool" and select "Luxtorpeda".
<img height="220px" src="https://user-images.githubusercontent.com/54072917/139227152-0536ac68-0d4b-44bf-be88-42105f5c3dd6.png" />

## Installation (debug build, from source)

0. Download the latest version of Rust: https://www.rust-lang.org/ and verify that openssl is installed on your system.

Debian, Ubuntu and variants

       $ sudo apt install libssl-dev
       
Fedora 

       $ sudo dnf install openssl-devel

Arch
       
       $ sudo pacman -S openssl rust
       
1. Close Steam.
2. Clone the repository, then use makefile to trigger `cargo build` and install:

       $ git clone https://github.com/luxtorpeda-dev/luxtorpeda.git
       $ cd luxtorpeda
       $ make user-install

3. Start Steam.
4. In game properties window select "Force the use of a specific Steam Play
   compatibility tool" and select "Luxtorpeda&nbsp;(dev)".

## Configuration

A configuration json file named `config.json` will be located in the luxtorpeda directory. It has the following paramters:

- host_url - This is used to determine where the packages.json file is located remotely, for use in automatic updates of this file.
- should_do_update - If this parameter is set to true, then the packages.json file will be updated automatically.
- use_controller - If this parameter is set to true, then attempts to access controllers through SDL2. Defaults to true.
- use_steam_controller - If this parameter is set to true, then attempts to connect to steam controller through USB interface. Defaults to true. If false, can interact with UI through normal steam controller desktop emulation.
- disable_default_confirm - Disables default engine confirmation dialog. Defaults to false. This can be done globally in the config.json by setting ```disable_default_confirm``` to true, or setting ```LUX_DISABLE_DEFAULT_CONFIRM=1 %command%``` in the launch options of a particular game. Setting ```LUX_DISABLE_DEFAULT_CONFIRM=0 %command%``` will enable the confirmation if the config variable is set to disabled for that particular game.

## User Packages Override

A ```~/.config/luxtorpeda/user-packages.json``` file can be created, which will allow custom package information without having to change the normal packages.json file. This file should have the same format as packages.json, but can have either new games or overrides to existing games. See https://github.com/luxtorpeda-dev/luxtorpeda/issues/65 for more information.

## User Interface

When a prompt appears from the client, it will accept input from controllers, keyboard or mouse. These prompts can include the engine chooser, progress indicator, error notices, and questions. The input works the following way:

### Keyboard and Mouse

* Keyboard and mouse are always supported, even if a controller is detected.
* Keyboard icons will appear in the buttons if no controllers are detected. Keyboard arrows can be used to navigate the choices.

### Controllers

* SDL2's SDL_GameController is used to detect and accept inputs from controllers, other than the steam controller, so any controller that supports that interface should work.
* Controller icons will appear in the buttons if a controller is detected.
    * Icons are only available for controllers in the testing list below, with it falling back to the Xbox controller icons if an unknown controller is detected.
    * Input with that controller should still work but the icons may be incorrect. If additional controller support is wanted, feel free to open an issue.
* The following controllers have been tested:
    * Xbox One Controller
    * PS4 Controller (PS5 Controller should work and use PS4 icons)
    * Switch Pro Controller (Will show icons)
    * Steam Controller (Direct USB connection and through dongle)
        * If a steam controller is detected, then a special interface is setup that connects to the steam controller directly, via the USB signals. This is because normal behavior is to emulate a keyboard and mouse and would not be possible to detect input the normal way.
        * This is best done inside Steam Big Picture mode, as the client uses the existence of the "Steam Virtual Gamepad" controller to detect if a steam controller is there.
        * When using the steam controller, the client is taking over control of the controller for the short time the client is running. Once the client is finished, it will release control and Steam should then re-connect to it.

### Steam Deck

* Thanks to help with testing from LiamD at GamingOnLinux, luxtorpeda works with the Steam Deck! See more information at https://www.gamingonlinux.com/2022/03/steam-deck-using-luxtorpeda-for-morrowind-warzone-2100-and-x-com/
* The steam deck's controller does not function in the luxtorpeda UI, but you can use the trackpad as a mouse for selections. https://github.com/luxtorpeda-dev/luxtorpeda/issues/130 is to track fixing that in the future.

## Supported titles

Just click "Play" and Luxtorpeda will download and install the package for you.
You need to select Luxtorpeda as a compatibility tool first, of course.

When you launch a game that supports multiple engines, a prompt will appear asking for the engine that should be downloaded and launched. Once the engine has been picked, a second prompt will ask if the engine should become the default. Launches after this if "Yes" is picked in this dialog will not ask for the engine again. A file can be deleted to restore the engine prompt for a particular game. The file will have the following format: `~/.config/luxtorpeda/<app_id>/default_engine_choice.txt`

To see a list of supported titles, go to https://luxtorpeda-dev.github.io

Want a specific game? 

Check [issues](https://github.com/luxtorpeda-dev/packages/issues) to see if we are working on it.

You can also make a package request by creating a [new issue](https://github.com/luxtorpeda-dev/packages/issues/new/choose)

You can also [create a package yourself](https://github.com/luxtorpeda-dev/packages/blob/master/docs/Creating_a_Package.md)

## Development

You can use `cargo` as with any Rust project; `make` serves only as a convenient
frontend for packaging and triggering longer `cargo` commands.
