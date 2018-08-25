extern crate nix;

use std::io::{self, Write};
use std::process::Command;
use std::ffi::CString;

use nix::unistd::{fork, ForkResult};
use nix::sys::wait::{wait, waitpid, WaitStatus, WaitPidFlag};

// need to add left side of pipe as optional argument
fn spawn_proc(pr: Option<&str>, pr_arr: Vec<&str>){
    match fork() {
        Ok(ForkResult::Parent { child, .. }) => {
            match wait() {
                Ok(v) => println!("working fine {:?}", v),
                Err(e) => println!("error executing child proc {}", e),
            }
            println!("Continuing execution in parent process, new child has pid: {}", child);
        }
        Ok(ForkResult::Child) => println!("I'm a new child process"),
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
