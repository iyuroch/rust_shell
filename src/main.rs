extern crate nix;

use std::io::{self, Write};
use std::ffi::CString;
use std::collections::VecDeque;
use std::os::unix::io::RawFd;

use nix::unistd::{fork, ForkResult, execvp, pipe, dup2, close};
use nix::sys::wait::{wait, waitpid, WaitPidFlag};

// TODO: print pwd
// TODO: globs
// TODO: add cd to this
// TODO: find autocompletion

fn spawn_proc(proc_argv: Option<&str>, mut pr_arr: &mut VecDeque<&str>, 
                read_end: Option<RawFd>, write_end: Option<RawFd>, is_shell: bool){
    match fork() {
        Ok(ForkResult::Parent { child, .. }) => {
            // check here if it's terminal - if not, waitpid with nohung should be used
            // or we can even completely ignore subprocess
            // we need to check if this is shell process or not
            if is_shell {
                waitpid(child, Some(WaitPidFlag::__WALL)).expect("Child died of some reason");
                // wait().expect("Child returned unexpected result");
            } else {
                // we connect part of pipe to stdin of process
                match write_end {
                    Some(v) => {
                        // might crash here
                        close(read_end.unwrap()).unwrap();
                        dup2(v, 1).unwrap();
                    },
                    None => {},
                }
                // we wait for all children to run, return error code with nohang
                // waitpid(child, Some(WaitPidFlag::WNOWAIT)).expect("Child died of some reason");
            }
            // println!("Continuing execution in parent process, new child has pid: {}", child);
        }
        Ok(ForkResult::Child) => {
            // we should think of handling redirections here
            // and parsing execve
            let str_argv = match proc_argv {
                Some(v) => v,
                None => return,
            };

            match read_end {
                    Some(v) => {
                        // might crash here
                        close(write_end.unwrap()).unwrap();
                        dup2(v, 0).unwrap();
                    },
                    None => {},
            }

            if pr_arr.len() > 0 {
                // connect pipe
                // will bite if something wrong
                let (read_end, write_end) = pipe().unwrap();
                spawn_proc(pr_arr.pop_front(), &mut pr_arr, Some(read_end), Some(write_end), false);
            }
            
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
            // Doesn't work with edge cases: "|", " | ", if empty command
            // we form vector of strings command that needs to be executed
            let mut pr_arr = stdin_buff.split(" | ")
                                .map(|x| x.trim())
                                .filter(|x| *x != "")
                                .collect::<VecDeque<_>>();
            spawn_proc(pr_arr.pop_front(), &mut pr_arr, None, None, true);
        }
        io::stdout().write(b"cmd >>").expect("Cannot print output");
        io::stdout().flush().expect("Cannot flush output");
        stdin_buff.clear();
    }
}