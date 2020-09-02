## Changelog for luxtorpeda-dev

### 22.0 (2020-09-02)

* [ZeroPointEnergy] Fix path of packages-temp.json when downloading the packages.json

### 21.0 (2020-08-29)

* Changed location of packages.json to ~/.cache/luxtorpeda/packages.json

### 20.2 (2020-08-24)

* Added good robot using ubuntu 18.04 container
* Updated openxray with fix for SDL full screen
* opengothic - Upgrade to v0.29
* dosbox-staging - Upgrade to 0.75.1

### 20.1 (2020-08-17)

* Updated openrct2 to 0.3.0

### 20.0 (2020-08-06)

* Add manual-download command for integration with other programs. Example: ```./luxtorpeda manual-download 3730```. STEAM_ZENITY environment variable should point to the path of system zenity if not running inside steam.
* Added scummvm as an engine option
* Added residualvm as an engine option
* Added choice of container for use in building
* Added openxray, using ubuntu 18.04 container
* Added initial dosbox-staging support

### 19.0 (2020-07-30)

* Change to clear engine choice if dialog canceled
* Check for return from setup, in case of errors
* Use steam zenity instead of dialog and zenity command. This should improve look of the zenity pop-ups.
* Added common QT 5.9 for use with engine building and usage.
* Added openmw launcher support using common qt
* Added openapoc launcher support using common qt
* Added warzone 2100 using common qt
* Added openmw-tes3mp support using common qt

### 18.0 (2020-07-30)

- Change to ask for engine choice before starting download. This will store in the ~/.config/luxtorpeda folder for the run command to use.

### 17.0 (2020-07-29)

- Added support for choices for engines
- Added support for default choice, when app id is not in packages.json
- Added openxcom
- Added openxcom-oxce

### 16.0 (2020-07-15)

- Moved packages to use github releases, instead of bintray
- Added support for tar.gz extraction
- Added ut99 (proprietary engine)

### 15.0 (2020-07-13)

- Fix for interactive setup, where zenity was unable to show EULA step.

### 14.0 (2020-07-13)

- Added support for zip file extraction, for use in MojoSetup extraction.
- Added prey 2006 (proprietary engine)

### 13.0 (2020-07-12)

- Added interactive setup to allow for presenting EULA, downloading installers, etc. 
- Added support for tar.bz2 file extraction.
- Added download_config metadata.
- Changed quake4 to use new interactive setup.
- Changed ut2004 to use new interactive setup.

### 11.0 (2020-07-11)

- Added use_original_command_directory package switch, to be used for certain games like UT2004 that have pathing issues.
- Added ut 2004 (proprietary engine)
  - Client will ask before taking any steps if you want to use this engine
  - Engine script will be downloaded, along with the following libraries to help improve game experience:
    - sdlcl - SDL 2 compatibility layer
    - openal - Needed for sound to work
    - libstdc++.so.5 - Needed for game to launch, most modern distros do not have this version installed by default.
  - Engine script will take care of presenting the original EULA, downloading the installer, extracting the data, and setting up. The process will ask for the cd key, which can be seen in the Steam client.

### 10.0 (2020-07-09)

- Fixed regression related to the steam download progress bar not appearing.
- Fixed regression related to slower download speeds.
- Added openapoc
- Added quake 4 (proprietary engine)
  - Client will ask before taking any steps if you want to use this engine
  - Engine script and override default configuration will be downloaded
  - Engine script will take care of presenting the original EULA, downloading the installer, extracting the data, and setting up. In this case, it'll also copy the cd key to the proper place, so it's just launch and go.

### 9.0 (2020-07-07)

- Support for new download parameter for common packages called "cache_by_name". This is useful for engines that can support multiple games with one archive, to lessen the amount of disk and network activity used.
- Changed engine package creation for the following games to use common packages:
  - dhewm3
  - arxlibertatis
  - gzdoom
  - yquake2
  - ioquake3
- Updated dxx-rebirth engine to use SDL2 instead of SDL1. Fixed an issue with playback of music files.
- Updated gzdoom engine to use 4.4.2, with included fluidsynth and GCC 9 build support. Newer version should also fix crashing issues.
- Updated dxx-rebirth to latest master, using the new GCC 9 build support.
- Added openrct2, using the new GCC 9 build system. Steam overlay needs to be disabled for this game.

### 8.0 (2020-07-06)

- Support for new download parameter for "copy_only". This is useful for artifacts that just need to be copied, instead of being extracted.
- Patch for Arx Libertatis engine to use borderless full screen mode.
- Added new engines
  - dxx-rebirth
  - ctp2

### 7.0 (2020-07-01)

- Switch reqwest to use rustls instead of using installed openssl. This should help with problems on certain distros and their openssl versions.

### 6.0 (2020-06-28)

- Support for dialog boxes for warning user about non-free engines

### 4.0 (2020-06-28)

- Support for updating packages.json automatically. This will check if the local hash matches what is up to date on the server. If it does not match, it'll download the latest version. This feature can be disabled using the new config.json file.

### 3.0 (2020-06-13)

- Added new engine
  - jk2mv

### 2.0 (2020-06-03)

- Support for GitHub Action for PRs so that new pull requests can be tested automatically.
- Support for GitHub Actions for luxtorpeda project and docker runtime project.
- Ported all possible engines from old method.
- Show licenses of engines in new HTML page.

### 1.0 (2020-05-30)

- Create packages repository to host all build scripts to packaging engines. This replaces the individual repostories that the base project uses.
- Support for GitHub Actions to automatically build the engines, push to bintray, and update the static website hosted on GitHub Pages.
- Minor updates to github-related issue and pull request templates.
- Added new engines
  - Freespace 2
  - AVP
  - OpenGothic
  - RBDoom-3-BFG
  - Julius
