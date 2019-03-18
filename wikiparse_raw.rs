#[macro_use]
extern crate lazy_static;

extern crate bzip2;
extern crate regex;

use std::env;
use std::fs::File;
use std::io::Write;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::BufRead;
use std::path::Path;
use std::time::Instant;

use bzip2::read::BzDecoder;
use regex::Regex;

#[derive(PartialEq, Eq, Copy, Debug, Clone)]
pub enum State {
    Null,
    Wiki,
    Site,
    Page,
    Write,
    Skip,
}

fn parse(fname_inp: &String, fname_out: &String) {
    let path_inp = Path::new(fname_inp);
    let file_inp = File::open(&path_inp).expect("Couldn't open input file");
    let bz2_reader = BzDecoder::new(file_inp);
    let buf_reader = BufReader::new(bz2_reader);

    let path_out = Path::new(fname_out);
    let file_obj = File::create(path_out).expect("Couldn't create output file");
    let mut file_out = BufWriter::new(file_obj);

    let then = Instant::now();

    let mut total = 0;
    let mut hits = 0;

    let mut state = State::Null;
    let mut store = vec![];

    lazy_static! {
        static ref ISTAG: Regex = Regex::new(r" *<").unwrap(); // filter first since most lines are non-tags
        static ref GETTAG: Regex = Regex::new(r" *<(/?[^ >]+)(?: |>)").unwrap();
        static ref GETID: Regex = Regex::new(r" *<id>([^<]*)</id>").unwrap();
    }

    for (i, res) in buf_reader.lines().enumerate() {
        let line = res.unwrap();
        if ISTAG.is_match(&line) {
            let ref line0 = &line.clone();
            let cap = GETTAG.captures(line0).unwrap();
            let tag = cap.get(1).unwrap().as_str();
            match (state, tag) {
                (State::Null, "mediawiki") => {
                    state = State::Wiki;
                    writeln!(file_out, "{}", line).unwrap();
                },

                (State::Wiki, "siteinfo") => {
                    state = State::Site;
                    writeln!(file_out, "{}", line).unwrap();
                },
                (State::Wiki, "page") => {
                    state = State::Page;
                    store.clear();
                    store.push(line);
                },
                (State::Wiki, "/mediawiki") => {
                    state = State::Null;
                    writeln!(file_out, "{}", line).unwrap();
                },

                (State::Site, "/siteinfo") => {
                    state = State::Wiki;
                    writeln!(file_out, "{}", line).unwrap();
                },
                (State::Site, _) => {
                    writeln!(file_out, "{}", line).unwrap();
                }

                (State::Page, "title") => {
                    store.push(line);
                },
                (State::Page, "ns") => {
                    store.push(line);
                },
                (State::Page, "id") => {
                    let cap = GETID.captures(&line).unwrap();
                    let sid = cap.get(1).unwrap().as_str();
                    let id = sid.parse::<u64>().unwrap();

                    total += 1;
                    if total % 1000 == 0 {
                        let dur = then.elapsed().as_secs();
                        println!("articles {}, matches {}, id {}, time {}", total, hits, id, dur);
                    }

                    if id == 2336430 {
                        state = State::Write;
                        for s in &mut store {
                            writeln!(file_out, "{}", s).unwrap();
                        }
                        writeln!(file_out, "{}", line).unwrap();
                    } else {
                        state = State::Skip;
                    }
                },

                (State::Write, "/page") => {
                    state = State::Wiki;
                    hits += 1;
                    writeln!(file_out, "{}", line).unwrap();
                },
                (State::Write, _) => {
                    writeln!(file_out, "{}", line).unwrap();
                }

                (State::Skip, "/page") => {
                    state = State::Wiki;
                },
                (State::Skip, _) => {},

                (s, t) => {
                    panic!("Unandled state: {:?}, {}", s, t);
                }
            }
        } else if state == State::Write || state == State::Site {
            writeln!(file_out, "{}", line).unwrap();
        } else if state == State::Skip {
        } else {
            panic!("Unexpected line ({}): {:?}, {}", i, state, line);
        }
    }

    println!("{}", total);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let fname_inp = &args[1];
    let fname_out = &args[2];
    println!("Parsing {} -> {}", fname_inp, fname_out);
    parse(fname_inp, fname_out);
}
