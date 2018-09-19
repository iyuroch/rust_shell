extern crate nix;

use std::io::{self, Write};
use std::ffi::CString;
use std::collections::VecDeque;
use std::os::unix::io::RawFd;
use std::process;

use nix::unistd::{fork, ForkResult, execvp, pipe, dup2, close, Pid};
use nix::sys::wait::{wait, waitpid, WaitPidFlag, WaitStatus};

// TODO: print pwd
// TODO: globs
// TODO: add cd to this!!
// TODO: find autocompletion
// TODO: iterator isntead of for loop

// as we pass fd to the children - they still remain opened in the parent process,
// those cannot be used in children. if we close them in parent - we cannot pass them to the children
// as they remain closed. we can reopen them in child process, or close at the very end

fn spawn_proc(str_argv: &str, fd_arr: VecDeque<Option<RawFd>>, last: Option<()>) {
    let (parent_write_end, read_end, write_end, child_read_end) 
    = (fd_arr[0], fd_arr[1], fd_arr[2], fd_arr[3]);
    match fork() {
        Ok(ForkResult::Parent { child, .. }) => {

            match parent_write_end {
                Some(v) => {
                    match close(v) {
                        Ok(_) => {},
                        Err(_e) => {},
                    }
                }
                None => {},
            }

            match read_end {
                Some(v) => {
                    match close(v) {
                        Ok(_) => {},
                        Err(_e) => {},
                    }
                }
                None => {},
            }

            match last {
                Some(_) => match waitpid(child, Option::None) {
                    Ok(_) => {
                        return;
                    },
                    Err(e) => println!("Error: {}", e)
                },
                None => {},
            }
            
            return;
        }
        Ok(ForkResult::Child) => {
            // we should think of handling redirections here
            // and parsing execve


            match parent_write_end {
                Some(v) => close(v).unwrap(),
                None => {},
            }

            match child_read_end {
                Some(v) => close(v).unwrap(),
                None => {},
            }

            match read_end {
                    Some(v) => {
                        dup2(v, 0).unwrap();
                    },
                    None => {},
            }

            match write_end {
                    Some(v) => {
                        dup2(v, 1).unwrap();
                    },
                    None => {},
            }

            // might crash if bad arguments
            let cstr_argv: Vec<_> = str_argv.split(" ")
                                    .map(|arg| CString::new(arg).unwrap())
                                    .collect();

            let raw_argv = cstr_argv.into_boxed_slice();

            match execvp(&raw_argv[0], &*raw_argv) {
                Ok(_) => {},
                Err(_) => {
                    println!("Cannot execute command");
                    process::exit(0x0100);
                }
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

            let mut fd_arr: VecDeque<_> = vec![None, None].into_iter().collect();
            let mut last: Option<()> = None;
            let mut first_child: Option<Pid> = None;

            while let Some(pr_args) = pr_arr.pop_front() {
                if pr_arr.len() == 0 {
                    last = Some(());
                    fd_arr.extend(&[None, None]);
                } else {
                    let (read_end, write_end) = pipe().unwrap();
                    fd_arr.extend(&[Some(write_end.clone()), Some(read_end.clone())]);
                }
                // println!("{:?}", fd_arr);
                spawn_proc(pr_args, fd_arr.clone(), last);
                fd_arr = fd_arr.split_off(2);
            }
            // spawn_proc(pr_arr.pop_front(), &mut pr_arr, None, None, true);
        }
        io::stdout().write(b"cmd >>").expect("Cannot print output");
        io::stdout().flush().expect("Cannot flush output");
        stdin_buff.clear();
    }
}