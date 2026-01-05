# Unofficial Zolo

A community-maintained Linux desktop application for managing multiple **Zalo Web** accounts with isolated profiles.

![Unofficial Zolo](https://upload.wikimedia.org/wikipedia/commons/e/e4/Zalo_logo_2017.svg)

> **Disclaimer**: This is an UNOFFICIAL project. It is not affiliated with, endorsed by, or sponsored by Zalo or VNG Corporation. This application simply loads the official Zalo Web interface in isolated web views.

## Features

- **Multi-Account Support**: Run multiple Zalo accounts simultaneously.
- **Complete Isolation**: Each account runs in a sandboxed profile with its own cookies, storage, and cache.
- **Dark Mode UI**: Modern, clean interface designed for desktop.
- **Privacy Focused**: Direct connection to Zalo Web. No tracking, no data collection, no JavaScript injection.
- **Lightweight**: Built with Rust (backend) and Tauri (frontend) for native performance.

## Installation

### AppImage (Recommended)
1. Download the latest `.AppImage` from the [Releases](https://github.com/your-username/unofficial-zolo/releases) page.
2. Make it executable:
   ```bash
   chmod +x unofficial-zolo_1.0.0_amd64.AppImage
   ```
3. Run it:
   ```bash
   ./unofficial-zolo_1.0.0_amd64.AppImage
   ```

### Building from Source

**Requirements:**
- Linux (Arch, Debian, Ubuntu, etc.)
- Rust & Cargo (v1.70+)
- Node.js & npm
- WebKitGTK and base development tools

**Dependencies (Ubuntu/Debian):**
```bash
sudo apt update
sudo apt install libwebkit2gtk-4.0-dev build-essential curl wget libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev
```

**Dependencies (Arch Linux):**
```bash
sudo pacman -Syu
sudo pacman -S webkit2gtk base-devel curl wget openssl appmenu-gtk-module gtk3 libappindicator-gtk3 librsvg libvips
```

**Build Steps:**
1. Clone the repository:
   ```bash
   git clone https://github.com/your-username/unofficial-zolo.git
   cd unofficial-zolo
   ```

2. Install frontend dependencies:
   ```bash
   npm install
   ```

3. Run in development mode:
   ```bash
   npm run tauri dev
   ```

4. Build release bundle (AppImage):
   ```bash
   npm run tauri build
   ```
   Output will be in `src-tauri/target/release/bundle/appimage/`.

## Architecture & Security

This application is designed with security and privacy as the top priority.

- **Core**: Rust backend handles system operations and profile management.
- **WebView**: Uses system WebKitGTK to render Zalo Web.
- **Isolation**: Profiles are stored in `~/.local/share/unofficial-zolo/profiles/{account_id}`.
- **No Injection**: The app does **NOT** inject any JavaScript into Zalo Web. Zalo's code runs exactly as it does in a standard browser.
- **No Interception**: Network traffic goes directly from the WebView to Zalo's servers.

## Proxy Support (Experimental)

The app supports saving proxy configurations per account.
**Note on Linux**: Due to limitations in Tauri v2 + WebKitGTK, per-WebView proxy settings may not apply reliably. For best results, use a system-wide proxy or VPN if you need network masking.

## License

MIT License - see [LICENSE](LICENSE) for details.

---
*Zalo and the Zalo logo are trademarks of VNG Corporation.*
