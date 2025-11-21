# hyprchoosy

<p align="center">
  <strong>Smart browser router for Hyprland</strong>
</p>

<p align="center">
  Route URLs to different browsers based on the originating application or URL hostname.
</p>

After switching to [Omarchy](https://omarchy.org/), I needed a [Choosy](https://choosy.app/) replacement. So I built it.

---

## ✨ Features

- **Client-based routing** - Open links from Slack in Chrome, Discord in Firefox, etc.
- **URL-based routing** - Route specific domains to specific browsers
- **Priority system** - Client rules take precedence over URL rules
- **Automatic client detection** - Walks up the process tree to find the originating application
- **XDG compliant** - Respects `$XDG_CONFIG_HOME` for configuration
- **Lightweight** - Fast binary with minimal dependencies
- **Hyprland native** - Built specifically for Hyprland desktop environment

## 📦 Installation

### Binary Releases

Download the latest release for your architecture from [GitHub Releases](https://github.com/filipechagas/hyprchoosy/releases):

**x86_64 (Intel/AMD) - Standard:**
```bash
wget https://github.com/filipechagas/hyprchoosy/releases/latest/download/hyprchoosy-amd64.tar.gz
tar xzf hyprchoosy-amd64.tar.gz
sudo mv hyprchoosy /usr/local/bin/
```

**x86_64 (Intel/AMD) - Static (musl):**
```bash
wget https://github.com/filipechagas/hyprchoosy/releases/latest/download/hyprchoosy-amd64-musl.tar.gz
tar xzf hyprchoosy-amd64-musl.tar.gz
sudo mv hyprchoosy /usr/local/bin/
```

**ARM64 - Standard:**
```bash
wget https://github.com/filipechagas/hyprchoosy/releases/latest/download/hyprchoosy-arm64.tar.gz
tar xzf hyprchoosy-arm64.tar.gz
sudo mv hyprchoosy /usr/local/bin/
```

**ARM64 - Static (musl):**
```bash
wget https://github.com/filipechagas/hyprchoosy/releases/latest/download/hyprchoosy-arm64-musl.tar.gz
tar xzf hyprchoosy-arm64-musl.tar.gz
sudo mv hyprchoosy /usr/local/bin/
```

### From Source

```bash
git clone https://github.com/filipechagas/hyprchoosy
cd hyprchoosy
cargo install --path .
```

## ⚙️ Configuration

### 1. Create config file

Create `~/.config/hyprchoosy/config.toml`:

```toml
[default]
browser = "firefox"

# Route links from Slack to Chrome
[work]
browser = "google-chrome-stable"
clients = ["slack"]

# Route GitHub and work domains to Firefox
[personal]
browser = "firefox"
url = ["github.com", "gitlab.com"]

# Route social media to a separate browser profile
[social]
browser = "brave"
url = ["twitter.com", "reddit.com", "youtube.com"]

# Route work tools from specific clients
[dev]
browser = "chromium"
clients = ["discord", "telegram"]
url = ["localhost", "127.0.0.1"]
```

### 2. Set as default browser

Set hyprchoosy as your default URL handler in Hyprland:

```bash
xdg-settings set default-web-browser hyprchoosy.desktop
```

Or create `~/.config/mimeapps.list`:

```ini
[Default Applications]
x-scheme-handler/http=hyprchoosy.desktop
x-scheme-handler/https=hyprchoosy.desktop
x-scheme-handler/chrome=hyprchoosy.desktop
text/html=hyprchoosy.desktop
application/x-extension-htm=hyprchoosy.desktop
application/x-extension-html=hyprchoosy.desktop
application/x-extension-shtml=hyprchoosy.desktop
application/xhtml+xml=hyprchoosy.desktop
application/x-extension-xhtml=hyprchoosy.desktop
application/x-extension-xht=hyprchoosy.desktop
```

### 3. Create desktop entry

Create `~/.local/share/applications/hyprchoosy.desktop`:

```desktop
[Desktop Entry]
Version=1.0
Name=Hyprchoosy
Comment=Smart browser router for Hyprland
Exec=/usr/local/bin/hyprchoosy %u
Terminal=false
Type=Application
MimeType=x-scheme-handler/http;x-scheme-handler/https;text/html;
Categories=Network;
NoDisplay=true
```

Then update the desktop database:

```bash
update-desktop-database ~/.local/share/applications/
```

## 📖 Usage

### Basic usage

```bash
hyprchoosy https://github.com
```

### Configuration options

**Environment variables:**

- `HYPRCHOOSY_CONFIG` - Override config file location

```bash
HYPRCHOOSY_CONFIG=~/my-config.toml hyprchoosy https://example.com
```

### Configuration syntax

**`[default]` section:**

- `browser` - Default browser command (default: `"firefox"`)

**Named rule sections:**

- `browser` - Browser command to use for this rule
- `clients` - List of client application names (partial match, case-insensitive)
- `url` - List of hostnames/domains to match

**Matching rules:**

1. **Client matching** - Checks if any client name contains the substring
   - `clients = ["slack"]` matches `slack`, `slack-desktop`, etc.

2. **URL matching** - Checks hostname equality or subdomain
   - `url = ["github.com"]` matches `github.com` and `*.github.com`

3. **Priority** - Client rules > URL rules > Default

## 🛠️ How it works

1. **Process detection** - Walks up the process tree to find the originating application
2. **Client matching** - Checks if the client matches any configured rules
3. **URL matching** - If no client match, checks the URL hostname
4. **Default fallback** - Uses default browser if no rules match
5. **Launch** - Spawns the selected browser detached from the current process

**Skipped processes** (when detecting client):

- `xdg-open`, `gio`
- Shell processes (`bash`, `zsh`, `fish`, etc.)
- System processes (`systemd`, `dbus-daemon`)
- Desktop portals (`xdg-desktop-portal*`)

## 🔧 Development

### Building

```bash
cargo build --release
```

### Testing

```bash
cargo test
```

### Local release build

```bash
./scripts/build-release.sh
```

This creates release artifacts for multiple architectures in the `release/` directory.

## 🤝 Contributing

Contributions are welcome! Please follow [Conventional Commits](https://www.conventionalcommits.org/) for commit messages.

**Commit examples:**

- `feat: add new feature` - New feature (bumps minor version)
- `fix: bug fix` - Bug fix (bumps patch version)
- `docs: update README` - Documentation changes
- `feat!: breaking change` - Breaking change (bumps major version)

### Release process

1. Commit changes using conventional commits
2. Push to `main` branch
3. Release Please creates a release PR automatically
4. Merge the PR to create a new release
5. GitHub Actions builds and publishes artifacts

## 📄 License

MIT License - see [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

Inspired by [Browserpass](https://github.com/browserpass/browserpass-extension) and [Finicky](https://github.com/johnste/finicky).

## 📝 Alternatives

- [xdg-utils](https://www.freedesktop.org/wiki/Software/xdg-utils/) - Basic XDG utilities
- [mimeo](https://xyne.archlinux.ca/projects/mimeo/) - More general MIME type handler
- [handlr](https://github.com/chmln/handlr) - Modern XDG utils replacement

---

<p align="center">Made with ❤️ for Hyprland users</p>
