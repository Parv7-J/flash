use std::{
    collections::HashMap,
    io::{self, Write, stdin},
    process,
};

use flash::{executor, lexer, parser, utils};

fn main() -> Result<(), ()> {
    unsafe {
        if libc::isatty(libc::STDIN_FILENO) == 1 {
            //handle error
            libc::setpgid(0, 0);
            libc::tcsetpgrp(libc::STDIN_FILENO, process::id() as i32);
        };
    };
    let shell_pgid = unsafe { libc::getpgid(0) };
    if shell_pgid == -1 {
        return Err(());
    }

    let mut execution_context = utils::ExecutionContext {
        variables: HashMap::new(),
        built_ins: utils::built_ins(),
        shell_pgid: shell_pgid as u32,
        last_exit_status: 0,
    };

    utils::ignore_signals();

    let mut input = String::new();

    loop {
        input.clear();
        print!("$ ");
        io::stdout().flush().unwrap();
        stdin().read_line(&mut input).map_err(|_| ())?;
        let tokens = lexer::tokenization(input.clone()).unwrap();
        println!("{tokens:?}");
        let command = parser::parse(tokens).unwrap();
        println!("{command:?}");
        match executor::execute(&command, &mut execution_context) {
            Ok(code) => {
                execution_context.last_exit_status = code;
            }
            Err(e) => {
                eprintln!("{e:?}");
            }
        };
    }
}
