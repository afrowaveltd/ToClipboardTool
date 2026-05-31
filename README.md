# to-clipboard 📋

Small terminal helper for copying command output to the OS clipboard — and for sending clipboard content back into commands.

`to-clipboard` is intentionally tiny: it is not a shell replacement, not a terminal emulator, and not a clipboard manager. It is a practical bridge between stdout, stdin, and the desktop clipboard.

---

## Quick start

```bash
cargo build --release
./target/release/to-clipboard echo "Hello World"
```

Default behavior:

```bash
to-clipboard <program> [args...]
```

The command runs directly. Its stdout is printed and also copied to the clipboard.

---

## Examples

```bash
# Print output and copy it to clipboard
to-clipboard echo "Hello World"

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

---

## Build

```bash
cargo build --release
```

Optional helper script:

```bash
./build.sh current
```

---

## License

MIT © 2026 Afrowave — see [LICENSE](LICENSE).
