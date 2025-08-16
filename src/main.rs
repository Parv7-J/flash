use std::{
    collections::HashMap,
    io::{self, Write, stdin},
    process,
};

use flash::{executor::execute, lexer::lexer, parser::parse, utils::ExecutionContext};

fn main() -> io::Result<()> {
    let mut input = String::new();

    loop {
        input.clear();
        print!("$ ");
        io::stdout().flush().unwrap();
        stdin().read_line(&mut input)?;
        let tokens = lexer(input.clone()).unwrap();
        // println!("{tokens:?}");
        let command = parse(tokens).unwrap();
        // println!("{command:?}");
        let pid = process::id();
        let pgid = unsafe { libc::getpgid(pid as i32) } as u32;
        let mut execution_context = ExecutionContext {
            variables: HashMap::new(),
            shell_pgid: pgid,
            last_exit_status: 0,
        };
        match execute(&command, &mut execution_context) {
            Ok(code) => {
                execution_context.last_exit_status = code;
            }
            Err(e) => {
                eprintln!("{e:?}");
            }
        };
    }
}
