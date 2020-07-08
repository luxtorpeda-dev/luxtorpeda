## Changelog for luxtorpeda-dev

### 9.0 (2020-07-07)

- Support for new download parameter for common packages called "cache_by_name". This is useful for engines that can support multiple games with one archive, to lessen the amount of disk and network activity used.
- Changed engine package creation for the following games to use common packages:
  - dhewm3
  - arxlibertatis
  - gzdoom
  - yquake2
  - ioquake3
- Updated dxx-rebirth engine to use SDL2 instead of SDL1. Fixed an issue with playback of music files.

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
