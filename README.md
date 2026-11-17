# Averon Fetch

A modern, lightweight system information utility written in Rust.

Developed by the [Averon Software Foundation](https://github.com/Averon-Software-Foundation).

Averon Fetch displays detailed system information in the terminal with a clean, structured layout. It is designed as an independent alternative to tools such as Fastfetch, with its own aesthetic and configuration system built from the ground up using Rust.

---

## Features

- Native Rust implementation with no external interpreters
- Modular information system: every row is produced by an independent module
- Enable, disable, reorder, and relabel modules through TOML configuration
- Automatic logo scaling and narrow-terminal handling
- 24-bit true colour, 256-colour fallback, and plain-text output
- Respects `NO_COLOR` and never emits ANSI sequences when piped by default
- JSON output for scripting and integration
- Linux-first design with native x86_64 and ARM64 (aarch64) support
- Cached system queries within a single execution
- Lightweight startup and minimal subprocess use

---

## Terminal Output Example

```
         ▄▄▄▄▄▄                josh@joshpc
       ▄█████████▄▄            ────────────────────
      ██████  ████████         OS              : CachyOS x86_64
     ████      ████████        Host            : B550 EAGLE
    ████        ▐████████      Kernel          : Linux 6.11.3
   ████           ████████     Uptime          : 4 hours, 23 minutes
  ████             ████████    Packages        : 1450 (pacman), 20 (flatpak)
 ████               ████████   Shell           : fish 3.7.0
 ████▄▄▄▄▄▄▄▄▄▄▄▄▄▄████████    Terminal        : konsole
  ████████████████████████     Display         : Acer XB273 GX 2560x1440 @ 165Hz
   ██████         ████████     Desktop         : KDE Plasma 6.2
    ████           ███████     WM              : KWin
     ██             ██████     Display Server  : Wayland
                   █████       CPU             : AMD Ryzen 7 5800X (8C/16T) @ 3.80 GHz
                    ████       CPU Temp        : 45.0 °C
                               GPU             : NVIDIA GeForce RTX 5060 Ti
                               Memory          : 8.51 GiB / 15.53 GiB (55%)
                               Swap            : 1.88 GiB / 15.53 GiB (12%)
                               Disk (/)        : 282.00 GiB / 620.00 GiB (45%)
                               Local IP        : 192.168.1.100 (eth0)
                               BIOS            : F14 (B550 EAGLE)
                               Locale          : en_GB.UTF-8
```

---

## Installation

### Prebuilt Binaries

Release archives are published for Linux on both x86_64 and ARM64. The
binaries are statically linked against musl, so they run on glibc and musl
distributions (including Alpine and older ARM boards) without matching a host
C library:

```
averon-fetch-<version>-linux-x86_64.tar.gz
averon-fetch-<version>-linux-aarch64.tar.gz
```

Download the archive matching your machine's architecture from the
[releases page](https://github.com/Averon-Software-Foundation/Averon-Fetch/releases),
then extract and install:

```sh
tar -xzf averon-fetch-<version>-linux-<arch>.tar.gz
sudo install -Dm755 averon-fetch-<version>-linux-<arch>/averon-fetch /usr/local/bin/averon-fetch
sudo install -Dm755 averon-fetch-<version>-linux-<arch>/av-fetch /usr/local/bin/av-fetch
```

### From Source

```sh
git clone https://github.com/Averon-Software-Foundation/Averon-Fetch.git
cd Averon-Fetch
cargo build --locked --release
sudo install -Dm755 target/release/averon-fetch /usr/local/bin/averon-fetch
sudo install -Dm755 target/release/av-fetch /usr/local/bin/av-fetch
```

Two binaries are built: `averon-fetch` and the shorter `av-fetch`. They are
identical apart from the command name, so install whichever you prefer.

### Cargo Install

The crate is not published on crates.io yet. Install directly from a local
clone or from Git instead:

```sh
# From a clone
cargo install --locked --path .

# Or directly from the repository
cargo install --locked --git https://github.com/Averon-Software-Foundation/Averon-Fetch
```

Both commands install the `averon-fetch` and `av-fetch` binaries.

### Arch Linux - AUR (planned)

```sh
yay -S averon-fetch
```

---

## Usage

The examples use `averon-fetch`; replace it with `av-fetch` for the shorter
command name.

```
averon-fetch                                     Display system information
averon-fetch --help                              Show help
averon-fetch --version                           Show version
averon-fetch --logo                              Display only the logo
averon-fetch --logo-only                         Alias for --logo
averon-fetch --no-logo                           Display information without the logo
averon-fetch --list-modules                      List modules and enabled state
averon-fetch --module cpu                        Show only CPU information
averon-fetch -m cpu -m memory                    Show CPU and memory in that order
averon-fetch -m cpu,memory                       Same, comma-separated
averon-fetch --all                               Show all modules, ignoring config visibility
averon-fetch --format json                       Structured JSON output
averon-fetch --format text                       Terminal text output (default)
averon-fetch --no-color                          Disable colour
averon-fetch --color always                      Force colour, even when piped
averon-fetch --color never                       Force plain output
averon-fetch --pipe                              Plain text output (no colour)
averon-fetch --noip                              Hide local IP addresses
averon-fetch --config                            Show configuration path and contents
averon-fetch --config /path/to/config.toml       Use a specific configuration file
averon-fetch --config-path                       Print the effective configuration path
averon-fetch --generate-config                   Write a documented default configuration
averon-fetch --show-errors                       Print collector diagnostics to stderr
```

Module names are case-insensitive. Common aliases are accepted, including
`window_manager` for `wm`, `ip` and `local_ip` for `network`, `motherboard` for
`bios`, `gpu_util` for `gpu_usage`, and `vram` for `gpu_vram`.

`--module` overrides config visibility, so a disabled module can still be
requested explicitly. `--all` ignores visibility but keeps the configured
order. When `--module` is used in text mode, the logo is hidden so only the
requested rows are printed.

---

## Configuration

The default configuration file is:

```
$XDG_CONFIG_HOME/averon-fetch/config.toml
```

If `XDG_CONFIG_HOME` is not set, the path is:

```
~/.config/averon-fetch/config.toml
```

`--config PATH` overrides the path for a single run, and supports a leading
`~`. To see which file would be used, run `averon-fetch --config-path`. To
print the effective path together with the current contents, run
`averon-fetch --config` without a value.

Generate a documented default file with:

```sh
averon-fetch --generate-config
```

This creates the file and its parent directory when missing. It never
overwrites an existing configuration; if the file is already present it
reports the path and exits successfully.

A complete example:

```toml
[general]
logo = true
colour = true
separator = ": "
show_errors = false

[logo]
width = 28
scale = 1.0
# custom_ascii = """
#   /\
#  /  \
# /____\
# """

[colours]
label = "#5B53EB"
value = "#DCDCDC"
separator = "#64D8F5"
accent = "#5B53EB"
header_user = "#DCDCDC"
header_host = "#5B53EB"
logo_primary = "#DCDCDC"
logo_accent = "#5B53EB"

[terminal]
# auto, always, or never
colour = "auto"
truecolour = true

[modules]
username = true
hostname = true
os = true
arch = true              # include the architecture suffix on the OS row
host = true
kernel = true
uptime = true
packages = true
shell = true
terminal = true
display = true
desktop = true
window_manager = true
display_server = true
theme = true
cpu = true
cpu_usage = false
cpu_temperature = true
gpu = true
gpu_usage = false
gpu_temperature = false
gpu_vram = false
memory = true
swap = true
disk = true
network = true
network_speed = false
bios = true
battery = true
audio = false
locale = true

# Modules not listed keep their built-in order after the listed ones.
order = ["os", "host", "kernel", "uptime", "cpu", "memory", "disk"]

[labels]
# Custom labels may use module ids or default labels.
os = "System"
"Local IP" = "IP"
Icons = "Icon Theme"
```

Logo width and scale are both supported. `scale = 2.0` doubles the built-in
logo, while `width = 40` sets a target width directly. When `scale` is set to
anything other than its default of `1.0`, it takes precedence over `width`.
A custom `custom_ascii` block replaces the built-in mark; its
width is measured from the longest line, and it is coloured with
`logo_primary` before any padding or truncation needed to fit the terminal.

`arch` is not a standalone row. It controls whether the architecture suffix
is appended to the OS row. The header modules (`username` and `hostname`) are
also toggled through `[modules]`, but they render in the header above the
divider rather than as labelled rows.

Labels can be customised in `[labels]` either by module id (`os`, `disk`,
`cpu_temperature`) or by the default label (`"Local IP"`, `Icons`). Matching
by default label is case-insensitive. Labels with spaces must be quoted in
TOML.

### Colour behaviour

Colour is resolved in this order:

1. `--pipe` and `--no-color` force plain output.
2. `--color never` forces plain output, and `--color always` forces colour,
   including when stdout is piped or `NO_COLOR` is set.
3. Otherwise (`auto`), `NO_COLOR`, `TERM=dumb`, non-TTY stdout, or
   `[general] colour = false` disable colour.
4. In `auto` mode, `[terminal] colour = "never"` disables colour and
   `[terminal] colour = "always"` keeps it enabled after step 3; it cannot
   override `--pipe`, `--no-color`, or the automatic checks.

Further details:

- `auto` (default) uses colour only on an interactive terminal.
- `NO_COLOR` disables automatic colour whenever it is present, even if it is
  empty. An explicit `--color always` still overrides it.
- `--color always` is the explicit opt-in that permits ANSI output when piped.
- `[terminal] colour` is consulted only in `auto` mode. Unlike `--color
  always`, it cannot enable colour when stdout is piped, `NO_COLOR` is set,
  `TERM=dumb`, or `[general] colour = false`.
- `truecolour = false` uses a 256-colour approximation.
- If the terminal does not advertise 24-bit support, `auto` uses 256 colours.
  `COLORTERM=truecolor` or `COLORTERM=24bit` enables true colour.
- `[general] colour = false` is a legacy switch that disables automatic
  colour; explicit `--color always` still wins.

---

## Supported Information

| Module | Information |
|--------|-------------|
| Username / Hostname | Header line above the divider |
| OS | Distribution, architecture |
| Kernel | Linux kernel version |
| Uptime | System uptime |
| Packages | Installed package counts per manager |
| Shell | Shell name and version |
| Terminal | Terminal emulator |
| Display | Every connected monitor, resolution, refresh rate |
| Desktop | Desktop environment and version |
| WM | Window manager |
| Display Server | Wayland or X11 |
| Theme | GTK theme, icon theme, cursor theme, font |
| CPU | Model, cores, threads, frequency |
| CPU Usage | Current CPU utilisation |
| CPU Temperature | CPU temperature from hwmon or thermal zones |
| GPU | Model and driver |
| GPU Usage | Utilisation from sysfs or `nvidia-smi` |
| GPU Temperature | Temperature from sysfs or `nvidia-smi` |
| GPU VRAM | Video memory usage |
| Memory | RAM usage |
| Swap | Swap usage |
| Disk | Mounted disk usage per partition |
| Network | Local IP addresses |
| Network Speed | Interface link speeds from sysfs |
| BIOS | Motherboard BIOS version |
| Battery | Charge level and status |
| Audio | ALSA devices and default PulseAudio/PipeWire sink |
| Locale | System locale and keyboard layout |

Modules only emit rows when the required information is available. Missing
hardware, unsupported drivers, and unreadable files are omitted rather than
shown with placeholder values.

Some values depend on the session and the available helpers:

- `Display` reads connector state from sysfs. Refresh rate comes from
  `xrandr`, so it is not reported under a pure Wayland session.
- `Disk` reads mount points from `/proc/self/mounts` and sizes through
  `statvfs`, so it does not need GNU `df` (which BusyBox lacks) and handles
  mount points containing spaces.
- `Network` reads addresses through `getifaddrs`, so it works without
  `iproute2`; the `ip` command is only used as a fallback. IPv6 link-local and
  loopback addresses are skipped when choosing the reported address.
- `Network` prefers IPv4 and falls back to the first IPv6 address when an
  interface has no IPv4 address.
- `CPU Temp`, `GPU Usage`, `GPU Temp`, and `GPU VRAM` require readable sysfs
  entries or a working `nvidia-smi`.
- `Desktop`, `WM`, and `Locale` inspect environment variables and common
  config files; unknown desktops may be reported by their raw
  `XDG_CURRENT_DESKTOP` value.

---

## Environment Variables

| Variable | Effect |
|----------|--------|
| `XDG_CONFIG_HOME` | Base directory for the default config path |
| `HOME` | Fallback config base and expansion target for `~` |
| `NO_COLOR` | Disable automatic colour output when present |
| `COLORTERM` | `truecolor` or `24bit` enables 24-bit colour in `auto` mode |
| `TERM` | `dumb` disables automatic colour output |
| `COLUMNS` | Overrides the detected terminal width |
| `SHELL` | Shell name and version for the `shell` module |
| `TERM_PROGRAM`, `TERMINAL`, `VTE_VERSION` | Terminal emulator detection |
| `XDG_CURRENT_DESKTOP`, `DESKTOP_SESSION`, `KDE_FULL_SESSION` | Desktop and window manager detection |

---

## Performance

Averon Fetch keeps work proportional to what is displayed:

- A single `sysinfo::System` snapshot is shared within an execution.
- Expensive collectors run only when a selected module needs them.
- Packages, GPU details, display, desktop, shell, locale, and audio are not
  queried when their modules are disabled.
- `nvidia-smi` is called at most once, and only when a selected GPU detail
  module needs data that sysfs could not provide.
- Network addresses are collected with `getifaddrs`; the `ip` command is only
  used as a fallback when that call fails.
- Terminal width is read with an ioctl instead of a `stty` subprocess.
- CPU usage uses a short `/proc/stat` sample and is disabled by default.

---

## JSON Output

```sh
averon-fetch --format json
```

```json
{
  "header": {
    "username": "josh",
    "hostname": "hostname"
  },
  "modules": [
    {
      "module": "os",
      "label": "OS",
      "value": "Averon Linux x86_64"
    },
    {
      "module": "memory",
      "label": "Memory",
      "value": "8.51 GiB / 15.53 GiB (55%)"
    }
  ]
}
```

JSON output contains no ANSI sequences and never includes logo artwork.
`header` is `null` when neither header module is selected or available, and
`label` reflects any custom labels from `[labels]`. Rows with an empty value
are omitted, matching text output. Module order follows the same ordering
rules as the terminal output.

---

## Package Manager Support

Package counts are read from the local package databases and tools listed
below. No package manager is required for Averon Fetch itself to run.

| Manager | Detection Method |
|---------|-----------------|
| pacman | `/var/lib/pacman/local/` |
| apt/dpkg | `/var/lib/dpkg/status` |
| rpm | `rpm -qa` |
| flatpak | `flatpak list` |
| snap | `/snap/` directory |

---

## Building from Source

**Requirements:** Rust 1.85 or later and Cargo. Using the latest stable
toolchain is recommended.

Builds run natively on x86_64 and ARM64 (aarch64) Linux. CI builds and tests
both architectures on matching runners, so no cross-compilation toolchain is
required. Published release binaries are built for the `*-unknown-linux-musl`
targets and then verified to be statically linked:

```sh
case "$(uname -m)" in
  x86_64)  target=x86_64-unknown-linux-musl ;;
  aarch64) target=aarch64-unknown-linux-musl ;;
  *) echo "unsupported architecture: $(uname -m)" >&2; exit 1 ;;
esac
rustup target add "$target"
cargo build --locked --release --target "$target"
```

The resulting binaries live under `target/<target>/release/`.

```sh
cargo build --locked              # debug build
cargo build --locked --release    # optimised release build
cargo test --locked --all-targets # run tests
cargo fmt --all --check           # check formatting
cargo clippy --locked --all-targets -- -D warnings
```

---

## Contributing

Contributions are welcome. Please follow the Averon Software Foundation project conventions when submitting patches or pull requests. Ensure all code is formatted with `cargo fmt` and passes `cargo clippy` before submission. Open an issue to discuss significant changes before beginning work.

---

## Licence

Averon Fetch is licensed under the **GNU General Public License v3.0**.

See the [LICENSE](LICENSE) file for full terms.
This is a short, non-authoritative summary: you may copy, distribute, and
modify the software as long as changes are tracked in the source files, and
any modified version distributed, including GPL-licensed code compiled into
it, is made available under the GPL together with build and install
instructions. This is not legal advice; see the LICENSE file for the full
terms.
