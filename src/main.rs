use std::io::{self, Write};

fn main() {
    let mut stdin_buff = String::new();
    io::stdout().write(b"cmd >>").expect("Cannot print output");
    io::stdout().flush().expect("Cannot flush output");

    loop {
        io::stdin().read_line(&mut stdin_buff).expect("Cannot read input");

        io::stdout().write(b"cmd >>").expect("Cannot print output");
        // io::stdout().write(stdin_buff.as_bytes()).expect("Cannot print output");
        io::stdout().flush().expect("Cannot flush output");
        stdin_buff.clear();
    }
    // expand globs
    // fork, execve with arguments
}
