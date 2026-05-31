# to-clipboard

`to-clipboard` is a small terminal helper that connects command output, piped stdin, and the operating system clipboard.

It has two main directions:

- **copy mode**: run a command or read piped stdin, show the text, and copy it to the clipboard
- **paste mode**: read the clipboard and print it or pipe it into another command

The default behavior executes commands directly, without a shell. This keeps arguments predictable and safer, especially when paths or values contain spaces.

Use `--shell` only when shell behavior is intentionally needed, for example pipes, redirects, globbing, variables, or command chaining.

On Windows, shell mode uses non-interactive PowerShell. Use PowerShell syntax, or call `cmd /C ...` explicitly when a command needs legacy Command Prompt behavior.

On Linux, copied text is kept available by a small internal background owner process after the main CLI command exits. This matches how X11 and Wayland clipboards expect short-lived command-line tools to behave.

## Basic examples

```bash
to-clipboard echo "Hello World"
echo "Hello World" | to-clipboard
to-clipboard -s cat /etc/hostname
to-clipboard --confirm ls -la
to-clipboard --shell "ls -la | grep txt | head"
```

## Paste examples

```bash
to-clipboard -p
to-clipboard -p grep error
to-clipboard -p wc -w
to-clipboard -p --shell "grep error | head"
```

## Release readiness

The project includes CI for formatting, clippy, Linux tests, and Windows tests. Tagged versions such as `v0.1.0` are built by the release workflow and attached to the matching GitHub Release.

Linux releases include a portable archive, a Debian package, an RPM package, and checksum files. Snap and Flatpak are good future candidates, but they need dedicated sandbox and clipboard testing before publishing.

## Afrowave note

This file is the localized project introduction. Future translations should be generated from this `en.md` file and placed next to it using language codes such as `cs.md`, `fr.md`, or `sw.md`.
