# to-clipboard documentation

## Purpose

`to-clipboard` is designed as a tiny cross-platform command-line bridge between terminal workflows and the desktop clipboard.

The tool is intentionally simple:

- capture stdout from a command
- capture piped stdin when no command is supplied
- optionally print it to the terminal
- copy it to the OS clipboard
- read clipboard content back to stdout or stdin of another command

## Command model

There are two execution modes for commands.

### Direct command mode

This is the default mode.

```bash
to-clipboard echo "Hello World"
to-clipboard cat "/path/with spaces/file.txt"
```

The first argument is treated as the executable. The remaining arguments are passed directly to that executable.

This mode does not interpret shell syntax. Pipes, redirects, variables, command chaining, and globbing are not handled here.

### Shell mode

Use `--shell` when shell features are required.

```bash
to-clipboard --shell "ls -la | grep txt | head"
to-clipboard --shell "echo $HOME"
```

On Unix-like systems, the tool prefers `/bin/bash` when present and falls back to `sh`.

On Windows, shell mode uses `powershell.exe` with `-NoLogo -NoProfile -NonInteractive -ExecutionPolicy Bypass -Command`.

Use PowerShell syntax in Windows shell mode:

```powershell
to-clipboard --shell "Get-ChildItem | Select-Object -First 10"
```

If legacy Command Prompt behavior is required, call it explicitly:

```powershell
to-clipboard --shell "cmd /C dir"
```

## Copy mode

Copy mode is the default.

```bash
to-clipboard <program> [args...]
```

Behavior:

1. Run the command.
2. Capture stdout.
3. Pass stderr through to the terminal.
4. Print stdout unless silent or confirm mode is active.
5. Copy stdout to the OS clipboard.
6. Return the wrapped command's exit code when the command fails.

If no command is supplied and stdin is piped, stdin becomes the copy source:

```bash
echo "Hello World" | to-clipboard
cat notes.txt | to-clipboard --confirm
```

In this form, the input is printed unless silent or confirm mode is active, then copied to the clipboard.

If no command is supplied and stdin is an interactive terminal, the tool exits with a usage error.

### Silent mode

```bash
to-clipboard -s cat /etc/hostname
```

Silent mode copies stdout to the clipboard but does not print stdout.

### Confirm mode

```bash
to-clipboard --confirm ls -la
```

Confirm mode does not print stdout. Instead, it prints a short confirmation message to stderr after the clipboard is updated.

## Paste mode

Paste mode reads text from the OS clipboard.

```bash
to-clipboard -p
```

If no command is supplied, clipboard text is printed to stdout.

```bash
to-clipboard -p grep error
```

If a command is supplied, clipboard text is written to the command's stdin.

## Exit codes

| Code | Meaning |
|------|---------|
| `0` | Success |
| `1` | General execution error or process terminated without a normal exit code |
| `2` | Clipboard read/write error |
| `64` | Usage error |
| wrapped command code | Returned when the wrapped command exits with a non-zero status |

## Clipboard limitations

The tool uses the Rust `arboard` crate for clipboard access.

On Linux, clipboard access usually requires a graphical session with X11 or Wayland support. On a headless SSH-only server, clipboard access may fail because there is no desktop clipboard available.

Linux clipboard contents are owned by the process that copied them. A short-lived CLI process can therefore lose the copied text as soon as it exits. To avoid that, `to-clipboard` starts a small internal background owner process on Linux after a successful copy. The owner process serves paste requests and exits automatically once another application replaces the clipboard contents.

Clipboard writes require valid UTF-8 text. The tool writes captured command output back to stdout as raw bytes, but it rejects invalid UTF-8 for clipboard writes instead of copying corrupted replacement text.

## Installation

Release assets are designed to support several installation styles.

### Debian and Ubuntu

```bash
sudo apt install ./to-clipboard_0.1.0_amd64.deb
```

### Fedora, RHEL, and openSUSE

```bash
sudo rpm -Uvh ./to-clipboard-0.1.0-1.x86_64.rpm
```

### Portable Linux archive

```bash
tar -xzf to-clipboard-0.1.0-linux-x86_64.tar.gz
sudo install -m 755 to-clipboard-0.1.0-linux-x86_64/bin/to-clipboard /usr/local/bin/to-clipboard
```

### Windows and macOS

Download the matching standalone binary and place it somewhere on `PATH`.

### Snap and Flatpak

Snap and Flatpak packages are not produced yet. They are possible future distribution channels, but clipboard access from sandboxed packages needs dedicated desktop testing before publishing.

## Build and release

Local release build:

```bash
cargo build --release
```

Run the freshly built binary directly when testing:

```bash
./target/release/to-clipboard echo "Hello World"
./target/release/to-clipboard -p
```

If `which to-clipboard` points to an older system installation, replace that installed binary before testing the command by name.

Helper script:

```bash
./build.sh current
./build.sh setup
./build.sh linux
./build.sh windows
./build.sh macos
./build.sh all
```

The helper script checks whether requested Rust targets are installed before building cross-target binaries. Non-native targets may also require platform linkers or SDKs.

For Windows GNU cross-builds, the helper script uses a temporary target directory by default. This avoids MinGW tooling failures when the repository path contains spaces. Set `TO_CLIPBOARD_TARGET_DIR` to override that location.

The repository includes two GitHub Actions workflows:

- `CI` checks formatting and clippy on Linux, and runs tests on both Linux and Windows.
- `Release` builds Linux, Windows, and macOS binaries when a tag matching `v*.*.*` is pushed.
- Linux release jobs also publish `.tar.gz`, `.deb`, `.rpm`, and checksum files.

Release flow:

```bash
git tag v0.1.0
git push origin v0.1.0
```

The release workflow uploads the generated binaries to the matching GitHub Release.

## Documentation convention

This project follows the Afrowave documentation style:

- root `README.md` is the public quick-start document
- `Readme/en.md` is the localized introductory readme source
- `Docs/en.md` is the localized documentation source

Future translation tooling can scan these folders recursively and generate localized variants next to the English source files.
