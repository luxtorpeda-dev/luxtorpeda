## Changelog for luxtorpeda-dev

* Packages changelog can be seen at https://github.com/luxtorpeda-dev/packages/blob/master/CHANGELOG.md

### 29.0 (2021-08-06)

* Add support for downloading progress dialog. This will show the amount of items being downloaded and the percentage of the download progress of the current item. This dialog will then disappear once all of the downloads are complete.
* Removed legacy way of communication with steam for the download process. Originally, Steam would call luxtorpeda twice, once for the download and setup, and once for the launch of the game. This appears to be removed in new steam installs, so now luxtorpeda will only respond to the launch game command and show a progress dialog created by luxtorpeda.
* Added error dialogs for issues such as download failing, not finding the package to launch, archive not extracting, etc.
* Fixed issue where Steam was attempting to launch overlay on every command luxtorpeda launches. Adding LD_PRELOAD="" seems to clean that up with no other issues. This should help with issues of too many file descripters and should make games that require installation scripts to load much faster the first time.

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
