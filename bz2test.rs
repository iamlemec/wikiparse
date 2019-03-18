extern crate bzip2;

use std::env;
use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use std::path::Path;
use bzip2::read::BzDecoder;

fn main() {
    let args: Vec<String> = env::args().collect();
    let fname_inp = &args[1];

    let path = Path::new(fname_inp);
    let file = File::open(&path).unwrap();

    let bz2_reader = BzDecoder::new(file);
    let buf_reader = BufReader::new(bz2_reader);

    let mut total = 0;
    let mut lines = 0;
    for line in buf_reader.lines() {
        match line {
            Err(e) => {
                panic!("Error (line {}, pos {}): {:?}", lines, total, e);
            },
            Ok(s) => {
                lines += 1;
                total += s.len() + 1;
            },
        }
    }
    total -= 1;

    println!("Total: {}", total);
}
