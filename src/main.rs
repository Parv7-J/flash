use std::{
    // ffi::CString,
    io::{self, Write, stdin},
};

use flash::{lexer, parse};

// const BUILT_INS: [&str; 2] = ["exit", "cd"];

fn main() -> io::Result<()> {
    let mut input = String::new();

    loop {
        input.clear();

        print!("$ ");
        io::stdout().flush().unwrap();

        stdin().read_line(&mut input)?;

        println!("{input}");

        let tokens = lexer(input.clone()).unwrap();

        println!("{tokens:?}");

        let command = parse(tokens).unwrap();

        println!("{command:?}");

        // let args = input.trim().split_whitespace().collect::<Vec<&str>>();

        // if args.is_empty() {
        //     continue;
        // }

        // let c_args = args
        //     .iter()
        //     .map(|arg| CString::new(*arg).unwrap())
        //     .collect::<Vec<CString>>();

        // let mut argv = c_args
        //     .iter()
        //     .map(|c_arg| c_arg.as_ptr())
        //     .collect::<Vec<*const libc::c_char>>();

        // argv.push(std::ptr::null());

        // if BUILT_INS.contains(&args[0]) {
        //     match args[0] {
        //         "exit" => std::process::exit(0),
        //         "cd" => {
        //             let path_to_go = match args.len() {
        //                 1 => match std::env::var("HOME") {
        //                     Ok(val) => Some(val),
        //                     Err(_) => {
        //                         eprintln!("cd: HOME not set");
        //                         None
        //                     }
        //                 },
        //                 2 => {
        //                     if args[1] == "~" {
        //                         match std::env::var("HOME") {
        //                             Ok(val) => Some(val),
        //                             Err(_) => {
        //                                 eprintln!("cd: HOME not set");
        //                                 None
        //                             }
        //                         }
        //                     } else {
        //                         Some(args[1].to_string())
        //                     }
        //                 }
        //                 _ => {
        //                     eprintln!("cd: too many arguments");
        //                     None
        //                 }
        //             };

        //             if let Some(path) = path_to_go {
        //                 let c_path = CString::new(path.clone()).unwrap();
        //                 unsafe {
        //                     if libc::chdir(c_path.as_ptr()) == -1 {
        //                         libc::perror(c_path.as_ptr());
        //                     }
        //                 }
        //             }
        //         }
        //         _ => {}
        //     }
        // } else {
        //     unsafe {
        //         let pid = libc::fork();
        //         if pid == -1 {
        //             eprintln!("fork failed");
        //             continue;
        //         }
        //         if pid == 0 {
        //             'inner: for (idx, arg) in args.iter().enumerate() {
        //                 if *arg == ">" {
        //                     if idx == args.len() - 1 {
        //                         eprintln!("No file name provided");
        //                         continue;
        //                     }
        //                     let filename = CString::new(args[idx + 1]).unwrap();
        //                     let filename_ptr = filename.as_ptr();

        //                     let open_flags = libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC;
        //                     let open_mode = 0o644;
        //                     let fd = libc::open(filename_ptr, open_flags, open_mode);

        //                     if fd == -1 {
        //                         libc::perror(filename_ptr);
        //                         continue;
        //                     }

        //                     libc::dup2(fd, 1);
        //                     libc::close(fd);

        //                     argv = argv[..idx].to_vec();
        //                     break 'inner;
        //                 }
        //             }

        //             libc::execvp(argv[0], argv.as_ptr());
        //             eprintln!("execvp failed for cmd: {}", args[0]);
        //             libc::exit(1);
        //         } else {
        //             let mut status = 0;
        //             libc::waitpid(pid, &mut status, 0);
        //         }
        //     }
        // }
    }
}
