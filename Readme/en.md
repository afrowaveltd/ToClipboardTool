# to-clipboard

`to-clipboard` is a small terminal helper that connects command output with the operating system clipboard.

It has two main directions:

- **copy mode**: run a command, show its output, and copy stdout to the clipboard
- **paste mode**: read the clipboard and print it or pipe it into another command

The default behavior executes commands directly, without a shell. This keeps arguments predictable and safer, especially when paths or values contain spaces.

Use `--shell` only when shell behavior is intentionally needed, for example pipes, redirects, globbing, variables, or command chaining.

## Basic examples

```bash
to-clipboard echo "Hello World"
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

## Afrowave note

This file is the localized project introduction. Future translations should be generated from this `en.md` file and placed next to it using language codes such as `cs.md`, `fr.md`, or `sw.md`.
