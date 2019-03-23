extern crate bzip2;
extern crate regex;
extern crate csv;

use std::env;
use std::fs::File;
use std::io::Write;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::BufRead;
use std::path::Path;
use std::time::Instant;
use std::collections::HashSet;

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

fn get_match(reg: &Regex, text: &str) -> String {
    let cap = reg.captures(text).unwrap();
    let mat = cap.get(1).unwrap().as_str();
    return String::from(mat);
}

fn parse(fname_inp: &String, fname_out: &String, id_set: HashSet<u64>) {
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
    let mut title = String::new();

    let mut state = State::Null;
    let mut store = vec![];

    let istag: Regex = Regex::new(r" *<").unwrap(); // filter first since most lines are non-tags
    let gettag: Regex = Regex::new(r" *<(/?[^ >]+)(?: |>)").unwrap();
    let getid: Regex = Regex::new(r" *<id>([^<]*)</id>").unwrap();
    let gettitle: Regex = Regex::new(r" *<title>([^<]*)</title>").unwrap();

    for (i, res) in buf_reader.lines().enumerate() {
        let mut line: String = res.unwrap();
        if istag.is_match(&line) {
            let tag = get_match(&gettag, &line);
            match (state, tag.as_str()) {
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
                    title = get_match(&gettitle, &line);
                    store.push(line);
                },
                (State::Page, "ns") => {
                    store.push(line);
                },
                (State::Page, "id") => {
                    let sid = get_match(&getid, &line);
                    let id = sid.parse::<u64>().unwrap();

                    total += 1;
                    if total % 1000 == 0 {
                        let dur = then.elapsed().as_secs();
                        println!("articles {}, matches {}, id {}, time {}", total, hits, id, dur);
                    }

                    if id_set.contains(&id) {
                        state = State::Write;
                        println!("{}", title);
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

// read in ids as a bool map
fn readlist(path: &String) -> HashSet<u64> {
    let mut rdr = csv::Reader::from_path(&path).unwrap();
    let _ = rdr.headers().unwrap();

    let mut ids: HashSet<u64> = HashSet::new();
    for res in rdr.records() {
        let rec = res.unwrap();
        let txt = rec.get(0).unwrap();
        let id = txt.parse::<u64>().unwrap();
        ids.insert(id);
    }

	return ids;
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let fname_inp = &args[1];
    let fname_out = &args[2];
    let fname_art = &args[3];
    println!("Parsing {} -> {}", fname_inp, fname_out);
    let id_set = readlist(fname_art);
    parse(fname_inp, fname_out, id_set);
}
