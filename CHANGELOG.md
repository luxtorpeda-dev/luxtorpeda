## Changelog for luxtorpeda-dev

* Packages changelog can be seen at https://github.com/luxtorpeda-dev/packages/blob/master/CHANGELOG.md

### 47.0 (2021-11-07)

* Fix for PS3 controller showing xbox glyphs.

### 46.0 (2021-11-05)

* Fix for user-packages default overriding all games.
* Add ```override_all_with_user_default``` flag to user-packages file, where if true, will override all games with the default in the user-packages file.
* Add support for providing the original executable path in environment variables, ```LUX_ORIGINAL_EXE``` and ```LUX_ORIGINAL_EXE_FILE```.
* Workaround for cargo bug for a dependency.

### 45.0 (2021-11-01)

Full release of egui UI. All features listed below are in this new release.

**New in 45 (2021-11-01)**
* Add support for showing additional details in engine choice menu. This is based on any notices that engine may have.

**New in 44 (2021-10-29)**
* Code cleanup.
* Fix crash with wayland, by removing opengl startup sequence that does not look to be needed.
* Add dialog for text input, currently used for ut2004. Allows controller to be used, by supporting a button for pasting into the input box.
* Support showing errors from run scripts inside the client with egui.

**New in 43 (2021-10-26)**
* Added basic management tool for clearing config and cache directories for particular games or engines. Can be accessed by going to the installation directory and executing ```./luxtorpeda.sh mgmt``` in a Terminal window.
* Added icons for Playstation controllers (will use the PS4 icons).
* Fix issue where using the joysticks on a connected controller while the client was running caused a freeze.
* When controller is lost, icons will fallback to keyboard icons. Hot-plugging is not supported.
* Added steam controller support, using direct USB access.
* Added scrolling with controller or keyboard to the scrollable prompts, such as the license agreement review.
* Add configuration parameters for disabling controller support or steam controller USB support.
* Support w and s for keyboard navigation of choices.

**New in 42 (2021-10-23)**
* Moved progress bar implementation from zenity to egui - See https://github.com/luxtorpeda-dev/luxtorpeda/pull/103 for futher information.
* Improved choices list to support controller friendly UI, using dpad to select items and buttons with controller icons. Keyboard navigation is also supported and will show icons for keyboard if no controller found. Arrow keys or w & s can be used to select items or can always use the mouse.
* Improved other UIs like progress, error & question prompts, and license agreements to have controller support
* Client-side decoration for title bar and SDL2 flags to make it more seamless
* Added default engine confirmation, where a window will appear for a few seconds to give the user a chance to clear the default and select a different engine.

**New in 41 (2021-10-20)**
* Moved GUI from gtk to egui with sdl2 backing
* For the controller support, steam virtual gamepad is disabled, then re-enabled before launching the game.
* For the new GUI, steam overlay is disabled, then re-enabled before launching the game.

### 40.0 (2021-10-17)

* Remove legacy support for original packages, so that runtime is now the default.

### 39.0 (2021-10-07)

* [Thanks to dfireBird] Log Client Version and Is Runtime At Launch
* [Thanks to dfireBird] Fix for (RS-W1015) Found string literal in env functions

### 37.0 (2021-09-27)

* Support Automatic Detection of Game Folders For Dependent Games - Detect game folders based on VDF files, for things like the source sdk games, so that manual picking is not required. Will fallback to original picking if not found.

### 36.0 (2021-09-24)

* Adds support for running games in Steam Runtime Soldier. With this version, there will be two compatibility tools, the original one named "Luxtorpeda" and one called "Luxtorpeda (Runtime"). Engines have been re-built for the new runtime. See https://luxtorpeda-dev.github.io/packages.html for the package list between the two, and the feature tracking ticket (https://github.com/luxtorpeda-dev/packages/issues/345) for more information.
* The original version will still work as normal and normal engine downloads will still work in non runtime mode.
* Support using rust-gtk, which gives greater control over the UI shown. Progress still uses zenity.

### 31.0 (2021-08-21)

* Fix issue in dialog detection for case of KDE Plasma and kdialog installed but qdbus not installed. In that case, it will fall back to zenity. If both are installed, it will use kdialog.

### 30.0 (2021-08-07)

* Fix issue in dialog detection for case of KDE Plasma with kdialog not installed. Now it will fall back to zenity. If kdialog is found, it will use that.

### 29.0 (2021-08-06)

* Add support for downloading progress dialog. This will show the amount of items being downloaded and the percentage of the download progress of the current item. This dialog will then disappear once all of the downloads are complete.
* Removed legacy way of communication with steam for the download process. Originally, Steam would call luxtorpeda twice, once for the download and setup, and once for the launch of the game. This appears to be removed in new steam installs, so now luxtorpeda will only respond to the launch game command and show a progress dialog created by luxtorpeda.
* Added error dialogs for issues such as download failing, not finding the package to launch, archive not extracting, etc.
* Fixed issue where Steam was attempting to launch overlay on every command luxtorpeda launches. Adding LD_PRELOAD="" seems to clean that up with no other issues. This should help with issues of too many file descriptors and should make games that require installation scripts to load much faster the first time.

### 28.0 (2021-08-03)

* Create generic dialog library for creation and processing of user interface.
* Add KDialog support for KDE Plasma based systems
* Add active_dialog_command parameter in config.json for default (pick the best for the running desktop), zenity, or kdialog
* Fix issue from last release where the license prompt could come up twice.

### 27.0 (2021-08-03)

* Support game launch running by itself without scripteval. Related to steam changes in regards to how luxtorpeda receives the commands from steam to download then launch the game. Now as a fallback the game launch command will also download the game files and ask for engine choices if the previous step was not done. See https://github.com/luxtorpeda-dev/luxtorpeda/issues/75 for more information.

### 26.0 (2021-08-03)

* Fix issue related to uninstall of a game set for luxtorpeda bringing up the game choice dialog. See https://github.com/luxtorpeda-dev/luxtorpeda/issues/47 for more information.

### 25.0 (2021-03-26)

* Add checks for download errors for packages
* Add support for user-packages.json. See https://github.com/luxtorpeda-dev/luxtorpeda/issues/65 for more information

### 24.0 (2021-02-14)

* New client release to support setting engine choice as default for games with multiple engines supported. README describes how it works and a file to delete to restore the engine choice prompt after picking a default.

### 23.0 (2020-09-23)

* Fix for ~/.cache/luxtorpeda folder not being created on first launch.

### 22.0 (2020-09-02)

* [ZeroPointEnergy] Fix path of packages-temp.json when downloading the packages.json

### 21.0 (2020-08-29)

* Changed location of packages.json to ~/.cache/luxtorpeda/packages.json

### 20.0 (2020-08-06)

* Add manual-download command for integration with other programs. Example: ```./luxtorpeda manual-download 3730```. STEAM_ZENITY environment variable should point to the path of system zenity if not running inside steam.

### 19.0 (2020-07-30)

* Change to clear engine choice if dialog canceled
* Check for return from setup, in case of errors
* Use steam zenity instead of dialog and zenity command. This should improve look of the zenity pop-ups.

### 18.0 (2020-07-30)

- Change to ask for engine choice before starting download. This will store in the ~/.config/luxtorpeda folder for the run command to use.

### 17.0 (2020-07-29)

- Added support for choices for engines
- Added support for default choice, when app id is not in packages.json

### 16.0 (2020-07-15)

- Moved packages to use github releases, instead of bintray
- Added support for tar.gz extraction

### 15.0 (2020-07-13)

- Fix for interactive setup, where zenity was unable to show EULA step.

### 14.0 (2020-07-13)

- Added support for zip file extraction, for use in MojoSetup extraction.

### 13.0 (2020-07-12)

- Added interactive setup to allow for presenting EULA, downloading installers, etc. 
- Added support for tar.bz2 file extraction.
- Added download_config metadata.

### 11.0 (2020-07-11)

- Added use_original_command_directory package switch, to be used for certain games like UT2004 that have pathing issues.

### 10.0 (2020-07-09)

- Fixed regression related to the steam download progress bar not appearing.
- Fixed regression related to slower download speeds.

### 9.0 (2020-07-07)

- Support for new download parameter for common packages called "cache_by_name". This is useful for engines that can support multiple games with one archive, to lessen the amount of disk and network activity used.

### 8.0 (2020-07-06)

- Support for new download parameter for "copy_only". This is useful for artifacts that just need to be copied, instead of being extracted.

### 7.0 (2020-07-01)

- Switch reqwest to use rustls instead of using installed openssl. This should help with problems on certain distros and their openssl versions.

### 6.0 (2020-06-28)

- Support for dialog boxes for warning user about non-free engines

### 4.0 (2020-06-28)

- Support for updating packages.json automatically. This will check if the local hash matches what is up to date on the server. If it does not match, it'll download the latest version. This feature can be disabled using the new config.json file.

### 2.0 (2020-06-03)

- Support for GitHub Action for PRs so that new pull requests can be tested automatically.
- Support for GitHub Actions for luxtorpeda project and docker runtime project.
- Ported all possible engines from old method.
- Show licenses of engines in new HTML page.

### 1.0 (2020-05-30)

- Create packages repository to host all build scripts to packaging engines. This replaces the individual repostories that the base project uses.
- Support for GitHub Actions to automatically build the engines, push to bintray, and update the static website hosted on GitHub Pages.
- Minor updates to github-related issue and pull request templates.
