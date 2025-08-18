use std::{collections::HashMap, ffi::CString, fs::File, os::fd::AsRawFd};

use libc::{STDIN_FILENO, STDOUT_FILENO};

use crate::utils::{
    Command, ConditionalType, ExecutionContext, ExecutionError, RedirectionType, SimpleCommand,
};

pub fn execute(
    node: &Command,
    context: &mut ExecutionContext,
    built_ins: &HashMap<
        String,
        Box<dyn Fn(SimpleCommand, &mut ExecutionContext) -> Result<i32, ExecutionError>>,
    >,
) -> Result<i32, ExecutionError> {
    match node {
        Command::Simple(sc) => {
            if let Some(closure) = built_ins.get(sc.command.as_str()) {
                return closure(sc.clone(), context);
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
        Command::Pipe { left, right } => unsafe {
            let mut pipe_fd = [0; 2];
            libc::pipe(pipe_fd.as_mut_ptr());

            let pid_left = libc::fork();

            if pid_left == -1 {
                return Err(ExecutionError::ForkFailed);
            } else if pid_left == 0 {
                libc::close(pipe_fd[0]);
                libc::dup2(pipe_fd[1], STDOUT_FILENO);
                libc::close(pipe_fd[1]);

                let exit_code = execute(left, context, built_ins).unwrap_or(1);

                libc::exit(exit_code);
            }

            let pid_right = libc::fork();

            if pid_right == -1 {
                return Err(ExecutionError::ForkFailed);
            } else if pid_right == 0 {
                libc::close(pipe_fd[1]);
                libc::dup2(pipe_fd[0], STDIN_FILENO);
                libc::close(pipe_fd[0]);

                let exit_code = execute(right, context, built_ins).unwrap_or(1);

                libc::exit(exit_code);
            }

            libc::close(pipe_fd[0]);
            libc::close(pipe_fd[1]);

            let mut status = 0;
            libc::waitpid(pid_left, &mut status, 0);
            libc::waitpid(pid_right, &mut status, 0);

            return Ok(status);
        },

        Command::Sequence { first, second } => {
            execute(first, context, built_ins)?;
            execute(second, context, built_ins)
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

            let status = execute(child_command, context, built_ins)?;

            unsafe {
                libc::dup2(saved_fd, std_fd);
                libc::close(saved_fd);
            }

            return Ok(status);
        }
        Command::Conditional {
            left,
            right,
            operator,
        } => {
            let exit_code = execute(left, context, built_ins).unwrap_or(1);
            if exit_code == 0 {
                match operator {
                    ConditionalType::And => {
                        return execute(right, context, built_ins);
                    }
                    ConditionalType::Or => {
                        return Ok(exit_code);
                    }
                }
            } else {
                match operator {
                    ConditionalType::And => {
                        return Ok(exit_code);
                    }
                    ConditionalType::Or => {
                        return execute(right, context, built_ins);
                    }
                }
            }
        }
        Command::Background { child_command } => unsafe {
            let pid = libc::fork();
            if pid == -1 {
                return Err(ExecutionError::ForkFailed);
            } else if pid == 0 {
                let null_fd = File::open("/dev/null")
                    .map_err(|e| ExecutionError::FileError(e))?
                    .as_raw_fd();
                libc::dup2(null_fd, libc::STDIN_FILENO);
                libc::close(null_fd);

                let exit_code = execute(child_command, context, built_ins).unwrap_or(1);

                libc::exit(exit_code);
            } else {
                context.jobs.push(pid);
                println!("[{}] {pid}", context.jobs.len());
                return Ok(0);
            }
        },
    }
}
