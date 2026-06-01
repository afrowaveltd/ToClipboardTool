# to-clipboard

Small terminal helper for copying command output or piped stdin to the OS clipboard, and for sending clipboard content back into commands.

`to-clipboard` is intentionally tiny: it is not a shell replacement, not a terminal emulator, and not a clipboard manager. It is a practical bridge between stdout, stdin, and the desktop clipboard.

---

## Quick start

```bash
cargo build --release
./target/release/to-clipboard echo "Hello World"
echo "Hello from stdin" | ./target/release/to-clipboard
```

Default behavior:

```bash
to-clipboard <program> [args...]
# or
some-command | to-clipboard
```

When a command is given, it runs directly. Its stdout is printed and also copied to the clipboard.

When no command is given and stdin is piped, stdin is printed and copied instead.

---

## Install

Download the matching asset from a GitHub Release.

Linux:

```bash
# Debian / Ubuntu
sudo apt install ./to-clipboard_0.1.0_amd64.deb

# Fedora / RHEL / openSUSE
sudo rpm -Uvh ./to-clipboard-0.1.0-1.x86_64.rpm

# Portable archive
tar -xzf to-clipboard-0.1.0-linux-x86_64.tar.gz
sudo install -m 755 to-clipboard-0.1.0-linux-x86_64/bin/to-clipboard /usr/local/bin/to-clipboard
```

Windows and macOS releases include standalone binaries. Put the binary somewhere on your `PATH`.

Snap and Flatpak packages are not shipped yet. Clipboard access from sandboxed packages needs extra desktop integration, so those formats should be added only after dedicated testing.

---

## Examples

```bash
# Print output and copy it to clipboard
to-clipboard echo "Hello World"

# Copy piped stdin
echo "Hello World" | to-clipboard

# Copy output only
to-clipboard -s cat /etc/hostname

# Copy output and show a short confirmation
to-clipboard --confirm ls -la

# Use shell features intentionally
to-clipboard --shell "ls -la | grep txt | head"

# Print clipboard content
to-clipboard -p

# Send clipboard content into a command
to-clipboard -p grep error
```

---

## Options

| Flag | Description |
|------|-------------|
| `-s`, `--silent` | Copy stdout to clipboard, print nothing |
| `-c`, `--confirm` | Copy stdout to clipboard and print a short confirmation to stderr |
| `-p`, `--paste` | Read from clipboard instead of capturing command stdout |
| `--shell` | Execute the command through the system shell |
| `-h`, `--help` | Show help |
| `-V`, `--version` | Show version |

---

## Direct mode vs shell mode

By default, `to-clipboard` executes commands directly:

```bash
to-clipboard cat "/path/with spaces/file.txt"
```

This keeps arguments predictable.

Shell syntax is available only when requested explicitly:

```bash
to-clipboard --shell "cat /var/log/syslog | grep error | head -5"
```

Use `--shell` for pipes, redirects, globbing, variables, and command chaining.

On Windows, `--shell` runs through non-interactive PowerShell. Use PowerShell syntax there, or call `cmd /C ...` explicitly when you need legacy `cmd.exe` behavior.

---

## Documentation

This repository follows the Afrowave documentation layout:

```text
README.md      # public quick start
Readme/en.md   # localized project introduction source
Docs/en.md     # localized documentation source
```

Future translations can be placed next to the English source files, for example:

```text
Readme/cs.md
Docs/cs.md
```

---

## Platform notes

The tool uses the Rust `arboard` crate for clipboard access.

On Linux, clipboard access usually requires a graphical X11 or Wayland session. On a headless SSH-only server, there may be no OS clipboard available.

Linux clipboards are owned by the process that copied the data. To keep copied text available after the main CLI process exits, `to-clipboard` starts a small internal background owner process on Linux. That process exits automatically after another application replaces the clipboard contents.

Clipboard text must be valid UTF-8. Command output is written back to stdout as raw bytes, but invalid UTF-8 is not copied to the clipboard.

---

## Build

```bash
cargo build --release
```

Run the freshly built binary directly:

```bash
./target/release/to-clipboard echo "Hello World"
./target/release/to-clipboard -p
```

If your shell resolves `to-clipboard` to an older installed binary, replace that installation with the new release binary.

Optional helper script:

```bash
./build.sh current
./build.sh windows
```

The helper script keeps Windows GNU cross-build artifacts in a temporary target directory by default. This avoids MinGW tooling failures when the project path contains spaces. Override it with `TO_CLIPBOARD_TARGET_DIR=/path/without/spaces` if needed.

## Release

The repository includes GitHub Actions workflows:

- `CI` runs formatting, tests, and clippy on pushes and pull requests to `main`.
- `Release` builds Linux, Windows, and macOS packages when a semantic version tag is pushed.
- Release assets include checksums for every package.

Release assets:

| Platform | Assets |
|----------|--------|
| Linux x86_64 | `.tar.gz`, `.deb`, `.rpm` |
| Windows x86_64 | `.zip` |
| macOS x86_64 | `.tar.gz` |
| macOS arm64 | `.tar.gz` |

Before tagging, run the local checks:

```bash
cargo fmt --check
cargo test --locked
cargo clippy --all-targets --all-features --locked -- -D warnings
```

Create a release by tagging a commit:

```bash
git tag -a v0.1.0 -m "Release v0.1.0"
git push origin v0.1.0
```

The release workflow creates the matching GitHub Release and uploads packaged assets.

---

## License

MIT (c) 2026 Afrowave - see [LICENSE](LICENSE).
