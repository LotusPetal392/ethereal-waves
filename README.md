# Ethereal Waves

A basic music player based on libcosmic. It's still very much a work in progress.

![Ethereal Waves - Dark Mode](https://github.com/LotusPetal392/ethereal-waves/blob/b970a4506b73b681b760d581c70f30d3a7eeed4b/screenshots/Ethereal%20Waves%20-%20Dark%20Mode.png?raw=true)
![Ethereal Waves - Light Mode](https://github.com/LotusPetal392/ethereal-waves/blob/b970a4506b73b681b760d581c70f30d3a7eeed4b/screenshots/Ethereal%20Waves%20-%20Light%20Mode.png?raw=true)

## Supported Formats
- MP3
- Ogg
- Opus
- Flac
- Wav

## Planned Features
Non-exhaustive list of planned features in no particular order:
- Gapless playback
- Grid view
- More column options in list view
- Export playlist as .m3u
- Improved MPRIS support
- Sorting options
- Shuffle modes
- Condensed responsive layout
- More keyboard shortcuts
- Drag and drop support
- Playlist duplicate management

## Keybindings
- `Ctrl + U`: Update Library
- `Ctrl + Q`: Quit
- `Ctrl + N`: New Playlist
- `F2`: Rename Playlist
- `Ctrl + Up`: Move Playlist Up
- `Ctrl + Down`: Move Playlist Down
- `Ctrl + =`: Zoom In
- `Ctrl + -`: Zoom Out
- `PageUp`: Scroll Up
- `PageDown`: Scroll Down
- `Ctrl + ,`: Settings
- `Ctrl + click`: Select
- `Ctrl + Shift + click`: Select Range

## Installation
This project uses `just` for building. To install it, run the following command:
```
cargo install just
```
Make sure ~/.cargo/bin is in your PATH. Run
```
just install
```
to install.
