# to-clipboard documentation

## Purpose

`to-clipboard` is designed as a tiny cross-platform command-line bridge between terminal workflows and the desktop clipboard.

The tool is intentionally simple:

- capture stdout from a command
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

On Windows, the tool uses PowerShell with `-NoProfile -Command`.

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

## Documentation convention

This project follows the Afrowave documentation style:

- root `README.md` is the public quick-start document
- `Readme/en.md` is the localized introductory readme source
- `Docs/en.md` is the localized documentation source

Future translation tooling can scan these folders recursively and generate localized variants next to the English source files.
