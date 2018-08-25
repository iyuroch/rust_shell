extern crate nix;

use std::io::{self, Write};
use std::ffi::CString;

use nix::unistd::{fork, ForkResult, execvp};
use nix::sys::wait::{wait};

// TODO: add cd to this
// TODO: find autocompletion

// need to add left side of pipe as optional argument
fn spawn_proc(proc_argv: Option<&str>, pr_arr: Vec<&str>){
    match fork() {
        Ok(ForkResult::Parent { child, .. }) => {
            // check here if it's terminal - if not, waitpid with nohung should be used
            // or we can even completely ignore subprocess
            wait().expect("Child returned unexpected result");
            // println!("Continuing execution in parent process, new child has pid: {}", child);
        }
        Ok(ForkResult::Child) => {
            // we should think of handling redirections here
            // and parsing execve
            // Catch unwrap here to handle empty arrays
            let str_argv = match proc_argv {
                Some(v) => v,
                None => return,
            };
            // might crash if bad arguments
            let cstr_argv: Vec<_> = str_argv.split(" ")
                                    .map(|arg| CString::new(arg).unwrap())
                                    .collect();
            let raw_argv = cstr_argv.into_boxed_slice();
            match execvp(&raw_argv[0], &*raw_argv) {
                Ok(_) => {},
                Err(_) => println!("Cannot execute command"),
            };
        },
        Err(_) => println!("Fork failed"),
    }
}

fn main() {
    let mut stdin_buff = String::new();
    io::stdout().write(b"cmd >>").expect("Cannot print output");
    io::stdout().flush().expect("Cannot flush output");

    loop {
        io::stdin().read_line(&mut stdin_buff).expect("Cannot read input");
        {
            // expand_glob();
            // Doesn't work with edge cases: "|", " | ", etc.
            let mut pr_arr = stdin_buff.split(" | ").map(|x| x.trim()).collect::<Vec<_>>();
            spawn_proc(pr_arr.pop(), pr_arr);
            // println!("{:?}", pr_arr)
        }
        // spawn();
        io::stdout().write(b"cmd >>").expect("Cannot print output");
        // io::stdout().write(stdin_buff.as_bytes()).expect("Cannot print output");
        io::stdout().flush().expect("Cannot flush output");
        stdin_buff.clear();
    }
    // expand globs
    // fork, execve with arguments
}
