use arboard::Clipboard;
use clap::Parser;
use std::io::{self, Write};
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

PASTE mode:
  Reads text from the OS clipboard and either prints it or sends it to a command via stdin.

By default, commands are executed directly without a shell.
Use --shell when you intentionally want shell features such as pipes, redirects, globbing,
variables, or command chaining.

Examples:
  to-clipboard echo 'Hello World'
  to-clipboard -s cat /etc/hostname
  to-clipboard --confirm ls -la
  to-clipboard --shell \"ls -la | grep txt | head\"

  to-clipboard -p
  to-clipboard -p grep error
  to-clipboard -p --shell \"grep error | head\"",
    after_help = "Happy piping! ツ"
)]
struct Cli {
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

fn main() {
    let cli = Cli::parse();

    if cli.paste {
        run_paste_mode(cli);
    } else {
        run_copy_mode(cli);
    }
}

fn run_copy_mode(cli: Cli) -> ! {
    if cli.command.is_empty() {
        eprintln!("to-clipboard: error: no command given");
        eprintln!("Try 'to-clipboard --help' for more information.");
        std::process::exit(EXIT_USAGE_ERROR);
    }

    let command_output = run_command_and_capture_stdout(&cli.command, cli.shell);

    let stdout_text = String::from_utf8_lossy(&command_output.stdout).to_string();

    if !cli.silent && !cli.confirm {
        print!("{}", stdout_text);
        flush_stdout_or_exit();
    }

    let mut clipboard_error = false;

    if stdout_text.is_empty() {
        if cli.confirm {
            eprintln!("Nothing copied: command produced no stdout.");
        }
    } else {
        match set_clipboard_text(&stdout_text) {
            Ok(()) => {
                if cli.confirm {
                    eprintln!("✔ Copied to clipboard: {}", preview_text(&stdout_text));
                }
            }
            Err(error_message) => {
                eprintln!("to-clipboard: error: {}", error_message);
                clipboard_error = true;
            }
        }
    }

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
    let mut command = Command::new("powershell");
    command.arg("-NoProfile").arg("-Command").arg(shell_command);
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

fn set_clipboard_text(text: &str) -> Result<(), String> {
    let mut clipboard =
        Clipboard::new().map_err(|error| format!("clipboard not available: {}", error))?;

    clipboard
        .set_text(text.to_string())
        .map_err(|error| format!("failed to set clipboard: {}", error))
}

fn flush_stdout_or_exit() {
    io::stdout().flush().unwrap_or_else(|error| {
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
        format!("{}…", truncated.escape_default())
    } else {
        truncated.escape_default().to_string()
    }
}
