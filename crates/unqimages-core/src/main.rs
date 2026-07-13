use std::io::{self, Write};

fn main() {
    io::stdout()
        .write_all(b"unqimages-core scaffold\n")
        .expect("write to stdout");
}
