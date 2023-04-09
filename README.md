# lfm-discord-presence
Discord Rich Presence for LastFM.

![](./assets/screenshot.png)

## Dependencies
- [Rust](https://www.rust-lang.org/)
- [Just](https://github.com/casey/just)

## Usage
### Setup
```bash
git clone https://git.cesium.pw/niku/lfm-discord-presence.git && cd $_
cp .env.sample .env
```
### Using systemd
Install:
```bash
just install
```
Uninstall:
```bash
just uninstall
```

### Using cargo
```bash
cargo run <lastfm-username>
```