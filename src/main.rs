extern crate nix;

use std::io::{self, Write};
use std::ffi::CString;
use std::collections::VecDeque;
use std::os::unix::io::RawFd;

use nix::unistd::{fork, ForkResult, execvp, pipe, dup2, close};
use nix::sys::wait::{wait, waitpid, WaitPidFlag, WaitStatus};

// TODO: print pwd
// TODO: globs
// TODO: add cd to this
// TODO: find autocompletion

fn spawn_proc(str_argv: &str, fd_arr: VecDeque<Option<RawFd>>){
    let (parent_write_end, read_end, write_end, child_read_end) 
                = (fd_arr[0], fd_arr[1], fd_arr[2], fd_arr[3]);
    match fork() {
        Ok(ForkResult::Parent { child, .. }) => {
            // we need to check if this is shell process or not
            match write_end {
                Some(_) => {
                    match waitpid(child, Some(WaitPidFlag::WNOHANG)) {
                        // need to match exit code of child here
                        Ok(_) => {},
                        Err(e) => println!("Error in child: {}", e),
                    }
                },
                None => {
                    match wait() {
                        Ok(_) => {},
                        Err(e) => println!("{}", )
                    }
                },
            }
            
            return;
        }
        Ok(ForkResult::Child) => {
            // we should think of handling redirections here
            // and parsing execve

            // match read_end {
            //         Some(v) => {
            //             // might crash here
            //             close(write_end.unwrap()).unwrap();
            //             dup2(v, 0).unwrap();
            //         },
            //         None => {},
            // }
            
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

            // in case of 3
            // {none(write_end), none(read_end), write-end, read-end, write-end, read-end, none}
            // (p1)<=>(p2)<=>(p3)
            // to close unused fd in child process we need to pass 2 pairs of pipes
            // can panic here!
            // let (write_end, read_end) = pipe().unwrap();
            // let mut fd_arr: VecDeque<_> = vec![None, None, Some(write_end.clone()), Some(read_end.clone())]
            //                     .into_iter().collect();

            let mut fd_arr: VecDeque<_> = vec![None, None].into_iter().collect();

            // println!("{:?}", fd_arr);

            while let Some(pr_args) = pr_arr.pop_front() {
                println!("{:?}", fd_arr);
                if pr_arr.len() == 0 {
                    fd_arr.extend(&[None, None]);
                } else {
                    let (write_end, read_end) = pipe().unwrap();
                    fd_arr.extend(&[Some(write_end.clone()), Some(read_end.clone())]);
                }
                spawn_proc(pr_args, fd_arr.clone());
                fd_arr = fd_arr.split_off(2);
            }
            // spawn_proc(pr_arr.pop_front(), &mut pr_arr, None, None, true);
        }
        io::stdout().write(b"cmd >>").expect("Cannot print output");
        io::stdout().flush().expect("Cannot flush output");
        stdin_buff.clear();
    }
}