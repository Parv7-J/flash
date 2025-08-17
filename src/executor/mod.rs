use std::{ffi::CString, fs::File, os::fd::AsRawFd};

use crate::utils::{Command, ExecutionContext, ExecutionError, RedirectionType};

pub fn execute(node: &Command, context: &mut ExecutionContext) -> Result<i32, ExecutionError> {
    match node {
        Command::Simple(sc) => {
            if let Some(closure) = context.built_ins.get(sc.command.as_str()) {
                return closure(sc.clone());
            }
            unsafe {
                let pid = libc::fork();
                if pid == -1 {
                    return Err(ExecutionError::ForkFailed);
                } else if pid == 0 {
                    let mut action: libc::sigaction = std::mem::zeroed();

                    action.sa_sigaction = libc::SIG_DFL;

                    let signals_to_reset = [
                        libc::SIGINT,
                        libc::SIGQUIT,
                        libc::SIGTSTP,
                        libc::SIGTTIN,
                        libc::SIGTTOU,
                    ];

                    for &signal in &signals_to_reset {
                        if libc::sigaction(signal, &action, std::ptr::null_mut()) == -1 {
                            panic!(
                                "Failed to reset signal handler in child for signal {}",
                                signal
                            );
                        }
                    }

                    let c_args = sc
                        .arguments
                        .iter()
                        .map(|arg| CString::new(arg.clone()).unwrap())
                        .collect::<Vec<CString>>();

                    let mut argv = c_args
                        .iter()
                        .map(|arg| arg.as_ptr())
                        .collect::<Vec<*const libc::c_char>>();

                    argv.push(std::ptr::null());

                    libc::execvp(argv[0], argv.as_ptr());

                    libc::exit(1);
                } else {
                    // libc::tcsetpgrp(libc::STDIN_FILENO, pid);

                    let mut status = 0;
                    libc::waitpid(pid, &mut status, 0);

                    // libc::tcsetpgrp(libc::STDIN_FILENO, context.shell_pgid as i32);
                    return Ok(status);
                }
            }
        }
        Command::Sequence { first, second } => {
            execute(first, context)?;
            execute(second, context)
        }
        Command::Redirect {
            child_command,
            redirect_type,
            target_file,
        } => {
            let (std_fd, saved_fd) = unsafe {
                match redirect_type {
                    RedirectionType::In => (libc::STDIN_FILENO, libc::dup(libc::STDIN_FILENO)),
                    _ => (libc::STDOUT_FILENO, libc::dup(libc::STDOUT_FILENO)),
                }
            };

            let file = match redirect_type {
                RedirectionType::In => match File::options().read(true).open(target_file) {
                    Ok(f) => f,
                    Err(e) => {
                        return Err(ExecutionError::FileError(e));
                    }
                },
                RedirectionType::Out => {
                    match File::options().create(true).write(true).open(target_file) {
                        Ok(f) => f,
                        Err(e) => {
                            return Err(ExecutionError::FileError(e));
                        }
                    }
                }
                RedirectionType::Append => {
                    match File::options().create(true).append(true).open(target_file) {
                        Ok(f) => f,
                        Err(e) => {
                            return Err(ExecutionError::FileError(e));
                        }
                    }
                }
            };

            let file_fd = file.as_raw_fd();

            std::mem::forget(file);

            unsafe {
                libc::dup2(file_fd, std_fd);
                libc::close(file_fd);
            }

            let status = execute(child_command, context)?;

            unsafe {
                libc::dup2(saved_fd, std_fd);
                libc::close(saved_fd);
            }

            return Ok(status);
        }
        _ => return Err(ExecutionError::Panic),
    }
}
