use std::{
    collections::HashMap,
    process::{ExitStatus, Stdio},
};

use tokio::process::Command;

use crate::models::ActiveRunner;

#[derive(serde::Deserialize, Debug)]
pub struct Runner {
    pub url: url::Url,
    pub provides: Option<Vec<String>>,
    pub start_script: String,
    pub stop_script: String,
    pub check_script: String,
}

impl Runner {
    pub async fn check_active(&self, me: &ActiveRunner) -> bool {
        let output = self.run_script(me, &self.check_script).await;
        output.status.success()
    }

    pub async fn start(&self, me: &ActiveRunner) -> Result<(), eyre::Error> {
        self.run_script_check(me, &self.start_script).await
    }

    pub async fn stop(&self, me: &ActiveRunner) -> Result<(), eyre::Error> {
        self.run_script_check(me, &self.stop_script).await
    }

    async fn run_script(&self, me: &ActiveRunner, script: &str) -> ShellOutput {
        let mut env = HashMap::new();
        if let Some(model) = &me.model {
            env.insert("MODEL".to_string(), model.to_string());
        }
        run_shell_script(&env, script).await
    }

    async fn run_script_check(&self, me: &ActiveRunner, script: &str) -> Result<(), eyre::Error> {
        let output = self.run_script(me, script).await;
        if output.status.success() {
            Ok(())
        } else {
            tracing::error!(
                stdout = output.stdout,
                stderr = output.stderr,
                script = script,
                status = output.status.code(),
                runner = me.name,
                "Running script failed"
            );

            eyre::bail!("Failed running script");
        }
    }
}

pub struct ShellOutput {
    pub status: ExitStatus,
    pub stdout: String,
    pub stderr: String,
}

async fn run_shell_script(environment: &HashMap<String, String>, script: &str) -> ShellOutput {
    let mut cmd = Command::new("/bin/sh");

    cmd.envs(environment)
        .stdin(Stdio::null())
        .arg("-c")
        .arg(script);

    tracing::debug!(cmd = ?cmd, "Executing");

    let output = cmd.output().await.expect("Missing /bin/sh");

    ShellOutput {
        status: output.status,
        // FIXME: use String::from_utf8_lossy_owned in the future
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[tokio::test]
    async fn running_shell_scripts_works() {
        let empty_env = HashMap::new();
        let putput = run_shell_script(&empty_env, "echo test").await;
        assert!(putput.status.success());
        assert_eq!(putput.stdout, "test\n");
    }

    #[tokio::test]
    async fn shell_script_env_works() {
        let mut me_env = HashMap::new();
        me_env.insert("MYVAR".to_string(), "test".to_string());
        let putput = run_shell_script(&me_env, "echo $MYVAR").await;
        assert!(putput.status.success());
        assert_eq!(putput.stdout, "test\n");
    }

    #[tokio::test]
    async fn running_shell_scripts_allows_bad_output() {
        let empty_env = HashMap::new();
        let putput = run_shell_script(&empty_env, "head -c 6 /dev/urandom").await;
        assert!(putput.status.success());
    }

    #[tokio::test]
    async fn correct_scripts_get_run() {
        let tmp_dir = tempfile::TempDir::new().unwrap();
        let stop_file = tmp_dir.path().join("stop");
        let start_file = tmp_dir.path().join("start");
        let check_file = tmp_dir.path().join("check");
        let runner = Runner {
            url: url::Url::parse("http://127.0.0.1").unwrap(),
            provides: None,
            start_script: format!("touch {}", start_file.to_str().unwrap()),
            stop_script: format!("touch {}", stop_file.to_str().unwrap()),
            check_script: format!("touch {}", check_file.to_str().unwrap()),
        };

        let me = ActiveRunner {
            name: "ollama".to_string(),
            model: None,
        };

        runner.stop(&me).await.unwrap();
        assert!(stop_file.exists());
        runner.start(&me).await.unwrap();
        assert!(start_file.exists());
        runner.check_active(&me).await;
        assert!(check_file.exists());
    }
}
