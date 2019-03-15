extern crate bzip2;
extern crate quick_xml;

use std::env;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use bzip2::read::BzDecoder;

use quick_xml::Reader;
use quick_xml::events::Event;

#[derive(PartialEq, Eq, Copy, Debug, Clone)]
pub enum State {
    Null,
    Wiki,
    Page,
    Title,
    Id,
    Revision,
    Text,
}

fn parse(fname_inp: &String) {
    let path = Path::new(fname_inp);
    let display = path.display();

    let file = match File::open(&path) {
        Err(e) => panic!("Couldn't open {}: {:?}", display, e),
        Ok(file) => file
    };

    let bz2_reader = BzDecoder::new(file);
    let buf_reader = BufReader::new(bz2_reader);
    let mut reader = Reader::from_reader(buf_reader);
    reader.trim_text(true);

    let mut total = 0;
    let mut state = State::Null;
    let mut buf = Vec::new();

    // The `Reader` does not implement `Iterator` because it outputs borrowed data (`Cow`s)
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => {
                match e.name() {
                    b"mediawiki" => {
                        if state == State::Null {
                            state = State::Wiki;
                        }
                    }
                    b"page" => {
                        if state == State::Wiki {
                            state = State::Page;
                        }
                    },
                    b"id" => {
                        if state == State::Page {
                            state = State::Id;
                        }
                    }
                    b"title" => {
                        if state == State::Page {
                            state = State::Title;
                        }
                    },
                    b"revision" => {
                        if state == State::Page {
                            state = State::Revision;
                        }
                    },
                    tag => {
                        if state == State::Revision {
                            // let text = e.unescape_and_decode(&reader).unwrap();
                            let text = String::from_utf8(tag.to_vec()).unwrap();
                            print!("<{}>", text);
                        }
                    },
                }
            },
            Ok(Event::End(ref e)) => {
                match e.name() {
                    b"mediawiki" => {
                        if state == State::Wiki {
                            state = State::Null;
                        }
                    }
                    b"page" => {
                        if state == State::Page {
                            state = State::Wiki;
                            total += 1;
                        }
                    },
                    b"id" => {
                        if state == State::Id {
                            state = State::Page;
                        }
                    }
                    b"title" => {
                        if state == State::Title {
                            state = State::Page;
                        }
                    },
                    b"revision" => {
                        if state == State::Revision {
                            state = State::Page;
                        }
                    },
                    tag => {
                        if state == State::Revision {
                            let text = String::from_utf8(tag.to_vec()).unwrap();
                            print!("</{}>\n", text);
                        }
                    },
                }
            },
            Ok(Event::Text(e)) => {
                match state {
                    State::Title => {
                        let text = e.unescape_and_decode(&reader).unwrap();
                        println!("Title: {}", text);
                    },
                    State::Id => {
                        let text = e.unescape_and_decode(&reader).unwrap();
                        println!("Id: {}", text);
                    },
                    _ => {
                        if state == State::Revision {
                            let text = e.unescape_and_decode(&reader).unwrap();
                            print!("{}", text);
                        }
                    },
                }
            },
            Ok(Event::Eof) => break, // exits the loop when reaching end of file
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            _ => (),
        }

        // if we don't keep a borrow elsewhere, we can clear the buffer to keep memory usage low
        buf.clear();
    }

    println!("{}", total);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let fname_inp = &args[1];
    println!("Parsing {}", fname_inp);
    parse(fname_inp);
}
