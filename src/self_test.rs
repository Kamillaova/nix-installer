use std::{process::Output, time::SystemTime};

use tokio::process::Command;
use which::which;

#[non_exhaustive]
#[derive(thiserror::Error, Debug, strum::IntoStaticStr)]
pub enum SelfTestError {
    #[error("Shell `{shell}` failed self-test with command `{command}`, stderr:\n{}", String::from_utf8_lossy(&output.stderr))]
    ShellFailed {
        shell: Shell,
        command: String,
        output: Output,
    },
    /// Failed to execute command
    #[error("Failed to execute command `{command}`",
        command = .command,
    )]
    Command {
        shell: Shell,
        command: String,
        #[source]
        error: std::io::Error,
    },
    #[error(transparent)]
    SystemTime(#[from] std::time::SystemTimeError),
}

#[cfg(feature = "diagnostics")]
impl crate::diagnostics::ErrorDiagnostic for SelfTestError {
    fn diagnostic(&self) -> String {
        let static_str: &'static str = (self).into();
        let context = match self {
            Self::ShellFailed { shell, .. } => vec![shell.to_string()],
            Self::Command { shell, .. } => vec![shell.to_string()],
            Self::SystemTime(_) => vec![],
        };
        format!(
            "{}({})",
            static_str,
            context
                .iter()
                .map(|v| format!("\"{v}\""))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Shell {
    Sh,
    Bash,
    Fish,
    Zsh,
}

impl std::fmt::Display for Shell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.executable())
    }
}

impl Shell {
    pub fn all() -> &'static [Shell] {
        &[Shell::Sh, Shell::Bash, Shell::Fish, Shell::Zsh]
    }
    pub fn executable(&self) -> &'static str {
        match &self {
            Shell::Sh => "sh",
            Shell::Bash => "bash",
            Shell::Fish => "fish",
            Shell::Zsh => "zsh",
        }
    }

    #[tracing::instrument(skip_all)]
    pub async fn self_test(&self) -> Result<(), SelfTestError> {
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    pub fn discover() -> Vec<Shell> {
        let mut found_shells = vec![];
        for shell in Self::all() {
            if which(shell.executable()).is_ok() {
                tracing::debug!("Discovered `{shell}`");
                found_shells.push(*shell)
            }
        }
        found_shells
    }
}

#[tracing::instrument(skip_all)]
pub async fn self_test() -> Result<(), Vec<SelfTestError>> {
    let shells = Shell::discover();

    let mut failures = vec![];

    for shell in shells {
        match shell.self_test().await {
            Ok(()) => (),
            Err(err) => failures.push(err),
        }
    }

    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures)
    }
}
