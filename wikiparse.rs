extern crate bzip2;
extern crate quick_xml;

use std::env;
use std::fs::File;
use std::io::Write;
use std::io::BufReader;
use std::io::BufRead;
use std::path::Path;
use std::time::Instant;

use bzip2::read::BzDecoder;

use quick_xml::Reader;
use quick_xml::events::Event;

#[derive(PartialEq, Eq, Copy, Debug, Clone)]
pub enum State {
    Null,
    Wiki,
    Site,
    Page,
    Write,
}

struct Page {
    title: String,
    ns: u64,
    id: u64,
}

/// Reads until end element is found consuming text as we go
/// Manages nested cases where parent and child elements have the same name
fn store_to_end<B: BufRead, K: AsRef<[u8]>>(reader: &mut Reader<B>, end: K, buf: &mut Vec<u8>) -> Result<(), String> {
    let mut depth = 0;
    let end = end.as_ref();
    loop {
        let pos = buf.len();
        match reader.read_event(&mut buf[pos..].to_vec()) {
            Ok(Event::End(ref e)) if e.name() == end => {
                if depth == 0 {
                    return Ok(());
                }
                depth -= 1;
            },
            Ok(Event::Start(ref e)) if e.name() == end => depth += 1,
            Ok(Event::Eof) => {
                return panic!("Expected </{:?}>", String::from_utf8(end.to_vec()));
            },
            Err(e) => panic!("Parsing error: {:?}", e),
            _ => (),
        }
    }
}

fn parse(fname_inp: &String, fname_out: &String) {
    let path_inp = Path::new(fname_inp);
    let file_inp = File::open(&path_inp).expect("Couldn't open input file");
    let bz2_reader = BzDecoder::new(file_inp);
    let buf_reader = BufReader::new(bz2_reader);
    let mut reader = Reader::from_reader(buf_reader);
    reader.trim_text(true);

    let path_out = Path::new(fname_out);
    let mut file_out = File::create(path_out).expect("Couldn't create output file");

    let then = Instant::now();

    let mut total = 0;
    let mut hits = 0;
    let mut state = State::Null;
    let mut buf = Vec::new();
    let mut txt = Vec::new();
    let mut page = Page { title: "".to_string(), ns: 0, id: 0};

    // The `Reader` does not implement `Iterator` because it outputs borrowed data (`Cow`s)
    loop {
        match (state, reader.read_event(&mut buf)) {
            (State::Null, Ok(Event::Start(ref e))) if e.name() == b"mediawiki" => {
                state = State::Wiki;
                write!(file_out, "<mediawiki>\n").unwrap();
            },

            (State::Wiki, Ok(Event::Start(ref e))) if e.name() == b"siteinfo" => {
                state = State::Site;
            },
            (State::Wiki, Ok(Event::Start(ref e))) if e.name() == b"page" => {
                state = State::Page;
                page.title = "".to_string();
                page.ns = 0;
                page.id = 0;
            },
            (State::Wiki, Ok(Event::End(ref e))) if e.name() == b"mediawiki" => {
                state = State::Null;
                write!(file_out, "</mediawiki>\n").unwrap();
            },

            (State::Site, Ok(Event::End(ref e))) if e.name() == b"siteinfo" => {
                state = State::Wiki;
            },
            (State::Site, _) => {}, // ignore siteinfo

            (State::Page, Ok(Event::Start(ref e))) if e.name() == b"title" => {
                page.title = reader.read_text(b"title", &mut txt).unwrap();
            },
            (State::Page, Ok(Event::Start(ref e))) if e.name() == b"ns" => {
                let text = reader.read_text(b"ns", &mut txt).unwrap();
                page.ns = text.parse::<u64>().unwrap();
            },
            (State::Page, Ok(Event::Start(ref e))) if e.name() == b"id" => {
                let text = reader.read_text(b"id", &mut txt).unwrap();
                page.id = text.parse::<u64>().unwrap();
                total += 1;
                if total % 50 == 0 {
                    let dur = then.elapsed();
                    println!("articles {}, matches {}, id {}, time {:?}", total, hits, page.id, dur);
                }
                if page.id == 2336430 {
                    state = State::Write;
                    write!(file_out, "<page>\n").unwrap();
                    write!(file_out, "<title>{}</title>\n", page.title).unwrap();
                    write!(file_out, "<ns>{}</ns>\n", page.ns).unwrap();
                    write!(file_out, "<id>{}</id>\n", page.id).unwrap();
                } else {
                    reader.read_to_end(b"page", &mut txt).unwrap();
                    state = State::Wiki;
                }
            },

            (State::Write, Ok(Event::End(ref e))) if e.name() == b"page" => {
                state = State::Wiki;
                hits += 1;
                write!(file_out, "</page>\n").unwrap();
                println!("{}: {}", hits, page.title);
            },
            (State::Write, Ok(Event::Start(e))) => {
                txt.clear();
                store_to_end(&mut reader, e.name(), &mut txt).unwrap();
                let tag = String::from_utf8(e.name().to_vec()).unwrap();
                let cont = String::from_utf8(txt.to_vec()).unwrap();
                write!(file_out, "<{}>", tag).unwrap();
                write!(file_out, "{}", cont).unwrap();
            },
            (State::Write, Ok(Event::End(e))) => {
            },
            (State::Write, Ok(Event::Empty(e))) => {
            },

            (State::Null, Ok(Event::Eof)) => break, // exits the loop when reaching end of file
            (s, e) => panic!("Unexpected state reached at {}: {:?}, {:?}", reader.buffer_position(), s, e),
        }

        // if we don't keep a borrow elsewhere, we can clear the buffer to keep memory usage low
        buf.clear();
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
