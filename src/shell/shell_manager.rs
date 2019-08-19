use crate::commands::command::EvaluatedWholeStreamCommandArgs;
use crate::errors::ShellError;
use crate::prelude::*;
use crate::shell::filesystem_shell::FilesystemShell;
use crate::shell::shell::Shell;
use crate::stream::OutputStream;
use std::error::Error;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub struct ShellManager {
    crate current_shell: usize,
    crate shells: Arc<Mutex<Vec<Box<dyn Shell + Send>>>>,
}

impl ShellManager {
    pub fn basic(commands: CommandRegistry) -> Result<ShellManager, Box<dyn Error>> {
        Ok(ShellManager {
            current_shell: 0,
            shells: Arc::new(Mutex::new(vec![Box::new(FilesystemShell::basic(
                commands,
            )?)])),
        })
    }

    pub fn push(&mut self, shell: Box<dyn Shell + Send>) {
        self.shells.lock().unwrap().push(shell);
        self.current_shell = self.shells.lock().unwrap().len() - 1;
        self.set_path(self.path());
    }

    pub fn pop(&mut self) {
        self.shells.lock().unwrap().pop();
        let new_len = self.shells.lock().unwrap().len();
        if new_len > 0 {
            self.current_shell = new_len - 1;
            self.set_path(self.path());
        }
    }

    pub fn is_empty(&self) -> bool {
        self.shells.lock().unwrap().is_empty()
    }

    pub fn path(&self) -> String {
        self.shells.lock().unwrap()[self.current_shell].path()
    }

    pub fn set_path(&mut self, path: String) {
        self.shells.lock().unwrap()[self.current_shell].set_path(path)
    }

    pub fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &rustyline::Context<'_>,
    ) -> Result<(usize, Vec<rustyline::completion::Pair>), rustyline::error::ReadlineError> {
        self.shells.lock().unwrap()[self.current_shell].complete(line, pos, ctx)
    }

    pub fn hint(&self, line: &str, pos: usize, ctx: &rustyline::Context<'_>) -> Option<String> {
        self.shells.lock().unwrap()[self.current_shell].hint(line, pos, ctx)
    }

    pub fn next(&mut self) {
        {
            let shell_len = self.shells.lock().unwrap().len();
            if self.current_shell == (shell_len - 1) {
                self.current_shell = 0;
            } else {
                self.current_shell += 1;
            }
        }
        self.set_path(self.path());
    }

    pub fn prev(&mut self) {
        {
            let shell_len = self.shells.lock().unwrap().len();
            if self.current_shell == 0 {
                self.current_shell = shell_len - 1;
            } else {
                self.current_shell -= 1;
            }
        }
        self.set_path(self.path());
    }

    pub fn ls(&self, args: EvaluatedWholeStreamCommandArgs) -> Result<OutputStream, ShellError> {
        let env = self.shells.lock().unwrap();

        env[self.current_shell].ls(args)
    }
    pub fn cd(&self, args: EvaluatedWholeStreamCommandArgs) -> Result<OutputStream, ShellError> {
        let env = self.shells.lock().unwrap();

        env[self.current_shell].cd(args)
    }
}
