use arboard::Clipboard;
#[cfg(target_os = "linux")]
use arboard::SetExtLinux;
use clap::Parser;
#[cfg(target_os = "linux")]
use std::env;
use std::io::{self, IsTerminal, Read, Write};
use std::process::{Command, ExitStatus, Stdio};

/// Runs a command, copies its stdout to the OS clipboard,
/// or sends clipboard content into a command.
#[derive(Parser, Debug)]
#[command(
    name = "to-clipboard",
    version,
    about = "Run a command and copy stdout to the OS clipboard, or pipe clipboard into a command.",
    long_about = "\
A small bidirectional clipboard bridge for your terminal.

COPY mode:
  Runs a command, prints stdout, and copies stdout to the OS clipboard.
  If no command is given and stdin is piped, reads stdin and copies it instead.

PASTE mode:
  Reads text from the OS clipboard and either prints it or sends it to a command via stdin.

By default, commands are executed directly without a shell.
Use --shell when you intentionally want shell features such as pipes, redirects, globbing,
variables, or command chaining.

Examples:
  to-clipboard echo 'Hello World'
  echo 'Hello World' | to-clipboard
  to-clipboard -s cat /etc/hostname
  to-clipboard --confirm ls -la
  to-clipboard --shell \"ls -la | grep txt | head\"

  to-clipboard -p
  to-clipboard -p grep error
  to-clipboard -p --shell \"grep error | head\"",
    after_help = "Happy piping!"
)]
struct Cli {
    /// Internal Linux clipboard owner process.
    #[cfg(target_os = "linux")]
    #[arg(long = "__to-clipboard-linux-daemon", hide = true)]
    linux_clipboard_daemon: bool,

    /// Silent mode: copy output to clipboard, print nothing.
    #[arg(short = 's', long = "silent", group = "copy_output_mode")]
    silent: bool,

    /// Confirm mode: copy silently, then print a short confirmation to stderr.
    #[arg(short = 'c', long = "confirm", group = "copy_output_mode")]
    confirm: bool,

    /// Paste mode: read from clipboard instead of running a command for stdout.
    /// If no command is given, prints clipboard content to stdout.
    /// If a command is given, pipes clipboard content into it as stdin.
    #[arg(short = 'p', long = "paste", conflicts_with_all = ["silent", "confirm"])]
    paste: bool,

    /// Execute the command through the system shell.
    /// Use this for pipes, redirects, globbing, variables, command chaining, etc.
    #[arg(long = "shell")]
    shell: bool,

    /// Command and arguments to execute.
    ///
    /// Without --shell:
    ///   to-clipboard echo hello
    ///
    /// With --shell:
    ///   to-clipboard --shell "echo hello | tr a-z A-Z"
    #[arg(num_args = 0.., trailing_var_arg = true, allow_hyphen_values = true)]
    command: Vec<String>,
}

const EXIT_GENERAL_ERROR: i32 = 1;
const EXIT_CLIPBOARD_ERROR: i32 = 2;
const EXIT_USAGE_ERROR: i32 = 64;
#[cfg(target_os = "linux")]
const LINUX_CLIPBOARD_DAEMON_ARG: &str = "--__to-clipboard-linux-daemon";
#[cfg(target_os = "linux")]
const LINUX_CLIPBOARD_DAEMON_READY: &[u8] = b"OK";

fn main() {
    let cli = Cli::parse();

    #[cfg(target_os = "linux")]
    if cli.linux_clipboard_daemon {
        run_linux_clipboard_daemon();
    }

    if cli.paste {
        run_paste_mode(cli);
    } else {
        run_copy_mode(cli);
    }
}

fn run_copy_mode(cli: Cli) -> ! {
    if cli.command.is_empty() {
        if io::stdin().is_terminal() {
            eprintln!("to-clipboard: error: no command given and no stdin piped");
            eprintln!("Try 'to-clipboard --help' for more information.");
            std::process::exit(EXIT_USAGE_ERROR);
        }

        let stdin_bytes = read_stdin_or_exit();
        let clipboard_error = copy_bytes_to_clipboard_and_maybe_stdout(
            &stdin_bytes,
            OutputMode::from_flags(cli.silent, cli.confirm),
        );

        std::process::exit(if clipboard_error {
            EXIT_CLIPBOARD_ERROR
        } else {
            0
        });
    }

    let command_output = run_command_and_capture_stdout(&cli.command, cli.shell);

    let clipboard_error = copy_bytes_to_clipboard_and_maybe_stdout(
        &command_output.stdout,
        OutputMode::from_flags(cli.silent, cli.confirm),
    );

    if !command_output.status.success() {
        std::process::exit(exit_code_from_status(command_output.status));
    }

    if clipboard_error {
        std::process::exit(EXIT_CLIPBOARD_ERROR);
    }

    std::process::exit(0);
}

fn run_paste_mode(cli: Cli) -> ! {
    let clipboard_text = match get_clipboard_text() {
        Ok(text) => text,
        Err(error_message) => {
            eprintln!("to-clipboard: error: {}", error_message);
            std::process::exit(EXIT_CLIPBOARD_ERROR);
        }
    };

    if cli.command.is_empty() {
        print!("{}", clipboard_text);
        flush_stdout_or_exit();
        std::process::exit(0);
    }

    let status = run_command_with_stdin(&cli.command, cli.shell, &clipboard_text);

    std::process::exit(exit_code_from_status(status));
}

struct CommandOutput {
    stdout: Vec<u8>,
    status: ExitStatus,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OutputMode {
    Print,
    Silent,
    Confirm,
}

impl OutputMode {
    fn from_flags(silent: bool, confirm: bool) -> Self {
        if silent {
            Self::Silent
        } else if confirm {
            Self::Confirm
        } else {
            Self::Print
        }
    }
}

fn read_stdin_or_exit() -> Vec<u8> {
    let mut input = Vec::new();
    io::stdin().read_to_end(&mut input).unwrap_or_else(|error| {
        eprintln!("to-clipboard: failed to read stdin: {}", error);
        std::process::exit(EXIT_GENERAL_ERROR);
    });
    input
}

fn copy_bytes_to_clipboard_and_maybe_stdout(bytes: &[u8], output_mode: OutputMode) -> bool {
    if output_mode == OutputMode::Print {
        write_stdout_or_exit(bytes);
    }

    match clipboard_text_from_bytes(bytes) {
        ClipboardText::Empty => {
            if output_mode == OutputMode::Confirm {
                eprintln!("Nothing copied: input produced no text.");
            }
            false
        }
        ClipboardText::Text(text) => match set_clipboard_text(text) {
            Ok(()) => {
                if output_mode == OutputMode::Confirm {
                    eprintln!("Copied to clipboard: {}", preview_text(text));
                }
                false
            }
            Err(error_message) => {
                eprintln!("to-clipboard: error: {}", error_message);
                true
            }
        },
        ClipboardText::InvalidUtf8 => {
            eprintln!("to-clipboard: error: clipboard text must be valid UTF-8");
            true
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
enum ClipboardText<'a> {
    Empty,
    Text(&'a str),
    InvalidUtf8,
}

fn clipboard_text_from_bytes(bytes: &[u8]) -> ClipboardText<'_> {
    if bytes.is_empty() {
        return ClipboardText::Empty;
    }

    match std::str::from_utf8(bytes) {
        Ok(text) => ClipboardText::Text(text),
        Err(_) => ClipboardText::InvalidUtf8,
    }
}

fn run_command_and_capture_stdout(command: &[String], use_shell: bool) -> CommandOutput {
    let mut process = build_command(command, use_shell);

    let output = process
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .output()
        .unwrap_or_else(|error| {
            eprintln!("to-clipboard: failed to run command: {}", error);
            std::process::exit(EXIT_GENERAL_ERROR);
        });

    CommandOutput {
        stdout: output.stdout,
        status: output.status,
    }
}

fn run_command_with_stdin(command: &[String], use_shell: bool, stdin_text: &str) -> ExitStatus {
    let mut child = build_command(command, use_shell)
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap_or_else(|error| {
            eprintln!("to-clipboard: failed to run command: {}", error);
            std::process::exit(EXIT_GENERAL_ERROR);
        });

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(stdin_text.as_bytes())
            .unwrap_or_else(|error| {
                eprintln!("to-clipboard: failed to write to command stdin: {}", error);
                std::process::exit(EXIT_GENERAL_ERROR);
            });
    }

    child.wait().unwrap_or_else(|error| {
        eprintln!("to-clipboard: failed to wait for command: {}", error);
        std::process::exit(EXIT_GENERAL_ERROR);
    })
}

fn build_command(command: &[String], use_shell: bool) -> Command {
    if use_shell {
        let shell_command = command.join(" ");
        return default_shell_command(&shell_command);
    }

    let mut process = Command::new(&command[0]);

    if command.len() > 1 {
        process.args(&command[1..]);
    }

    process
}

#[cfg(target_os = "windows")]
fn default_shell_command(shell_command: &str) -> Command {
    let mut command = Command::new("powershell.exe");
    command
        .arg("-NoLogo")
        .arg("-NoProfile")
        .arg("-NonInteractive")
        .arg("-ExecutionPolicy")
        .arg("Bypass")
        .arg("-Command")
        .arg(shell_command);
    command
}

#[cfg(not(target_os = "windows"))]
fn default_shell_command(shell_command: &str) -> Command {
    let shell_path = if std::path::Path::new("/bin/bash").exists() {
        "/bin/bash"
    } else {
        "sh"
    };

    let mut command = Command::new(shell_path);
    command.arg("-c").arg(shell_command);
    command
}

fn get_clipboard_text() -> Result<String, String> {
    let mut clipboard =
        Clipboard::new().map_err(|error| format!("clipboard not available: {}", error))?;

    clipboard
        .get_text()
        .map_err(|error| format!("failed to read clipboard: {}", error))
}

#[cfg(not(target_os = "linux"))]
fn set_clipboard_text(text: &str) -> Result<(), String> {
    let mut clipboard =
        Clipboard::new().map_err(|error| format!("clipboard not available: {}", error))?;

    clipboard
        .set_text(text.to_string())
        .map_err(|error| format!("failed to set clipboard: {}", error))
}

#[cfg(target_os = "linux")]
fn set_clipboard_text(text: &str) -> Result<(), String> {
    spawn_linux_clipboard_daemon(text)
}

#[cfg(target_os = "linux")]
fn run_linux_clipboard_daemon() -> ! {
    let mut text = String::new();

    if let Err(error) = io::stdin().read_to_string(&mut text) {
        eprintln!(
            "to-clipboard: error: failed to read Linux clipboard daemon stdin: {}",
            error
        );
        std::process::exit(EXIT_GENERAL_ERROR);
    }

    if let Err(error) = set_clipboard_text_once(&text) {
        eprintln!("to-clipboard: error: {}", error);
        std::process::exit(EXIT_CLIPBOARD_ERROR);
    }

    if io::stdout()
        .write_all(LINUX_CLIPBOARD_DAEMON_READY)
        .and_then(|_| io::stdout().flush())
        .is_err()
    {
        std::process::exit(EXIT_GENERAL_ERROR);
    }

    if let Err(error) = set_clipboard_text_and_wait(&text) {
        eprintln!("to-clipboard: error: {}", error);
        std::process::exit(EXIT_CLIPBOARD_ERROR);
    }

    std::process::exit(0);
}

#[cfg(target_os = "linux")]
fn spawn_linux_clipboard_daemon(text: &str) -> Result<(), String> {
    let mut child = Command::new(
        env::current_exe()
            .map_err(|error| format!("failed to find current executable: {}", error))?,
    )
    .arg(LINUX_CLIPBOARD_DAEMON_ARG)
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::inherit())
    .current_dir("/")
    .spawn()
    .map_err(|error| format!("failed to start Linux clipboard daemon: {}", error))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(text.as_bytes())
            .map_err(|error| format!("failed to send text to clipboard daemon: {}", error))?;
    }

    let mut ready = [0; 2];

    if let Some(mut stdout) = child.stdout.take() {
        stdout.read_exact(&mut ready).map_err(|error| {
            format!(
                "Linux clipboard daemon failed before setting clipboard: {}",
                error
            )
        })?;
    } else {
        return Err("failed to read Linux clipboard daemon status".to_string());
    }

    if ready == LINUX_CLIPBOARD_DAEMON_READY {
        Ok(())
    } else {
        Err("Linux clipboard daemon returned an invalid status".to_string())
    }
}

#[cfg(target_os = "linux")]
fn set_clipboard_text_once(text: &str) -> Result<(), String> {
    let mut clipboard =
        Clipboard::new().map_err(|error| format!("clipboard not available: {}", error))?;

    clipboard
        .set_text(text.to_string())
        .map_err(|error| format!("failed to set clipboard: {}", error))
}

#[cfg(target_os = "linux")]
fn set_clipboard_text_and_wait(text: &str) -> Result<(), String> {
    let mut clipboard =
        Clipboard::new().map_err(|error| format!("clipboard not available: {}", error))?;

    clipboard
        .set()
        .wait()
        .text(text.to_string())
        .map_err(|error| format!("failed to keep clipboard text available: {}", error))
}

fn flush_stdout_or_exit() {
    io::stdout().flush().unwrap_or_else(|error| {
        eprintln!("to-clipboard: failed to write stdout: {}", error);
        std::process::exit(EXIT_GENERAL_ERROR);
    });
}

fn write_stdout_or_exit(bytes: &[u8]) {
    let mut stdout = io::stdout().lock();
    stdout.write_all(bytes).unwrap_or_else(|error| {
        eprintln!("to-clipboard: failed to write stdout: {}", error);
        std::process::exit(EXIT_GENERAL_ERROR);
    });
    stdout.flush().unwrap_or_else(|error| {
        eprintln!("to-clipboard: failed to write stdout: {}", error);
        std::process::exit(EXIT_GENERAL_ERROR);
    });
}

fn exit_code_from_status(status: ExitStatus) -> i32 {
    status.code().unwrap_or(EXIT_GENERAL_ERROR)
}

/// Create a preview of the clipboard text: first line, max 60 chars.
fn preview_text(text: &str) -> String {
    let first_line = text.lines().next().unwrap_or("");
    let truncated: String = first_line.chars().take(60).collect();

    if first_line.chars().count() > 60 {
        format!("{}...", truncated.escape_default())
    } else {
        truncated.escape_default().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn clap_configuration_is_valid() {
        Cli::command().debug_assert();
    }

    #[test]
    fn output_mode_prefers_silent_over_confirm() {
        assert_eq!(OutputMode::from_flags(false, false), OutputMode::Print);
        assert_eq!(OutputMode::from_flags(true, false), OutputMode::Silent);
        assert_eq!(OutputMode::from_flags(false, true), OutputMode::Confirm);
        assert_eq!(OutputMode::from_flags(true, true), OutputMode::Silent);
    }

    #[test]
    fn clipboard_text_rejects_invalid_utf8() {
        assert_eq!(clipboard_text_from_bytes(b""), ClipboardText::Empty);
        assert_eq!(
            clipboard_text_from_bytes("hello".as_bytes()),
            ClipboardText::Text("hello")
        );
        assert_eq!(
            clipboard_text_from_bytes(&[0xff, 0xfe]),
            ClipboardText::InvalidUtf8
        );
    }

    #[test]
    fn preview_uses_first_line_and_escapes_control_chars() {
        assert_eq!(preview_text("hello\nworld"), "hello");
        assert_eq!(preview_text("hello\tworld"), "hello\\tworld");
    }

    #[test]
    fn preview_truncates_long_text() {
        let text = "a".repeat(61);
        assert_eq!(preview_text(&text), format!("{}...", "a".repeat(60)));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn windows_shell_command_uses_noninteractive_powershell() {
        let command = default_shell_command("Write-Output hello");
        let args: Vec<_> = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();

        assert_eq!(command.get_program().to_string_lossy(), "powershell.exe");
        assert_eq!(
            args,
            vec![
                "-NoLogo",
                "-NoProfile",
                "-NonInteractive",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                "Write-Output hello",
            ]
        );
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn unix_shell_command_uses_command_flag() {
        let command = default_shell_command("echo hello");
        let args: Vec<_> = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();

        assert_eq!(args, vec!["-c", "echo hello"]);
    }
}
