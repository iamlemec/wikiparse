extern crate bzip2;
extern crate quick_xml;

use std::env;
use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use std::path::Path;

use bzip2::read::BzDecoder;

use quick_xml::Reader;
use quick_xml::events::Event;

/* just cat out data */

fn cat(fname_inp: &String) {
    let path = Path::new(fname_inp);
    let display = path.display();

    let file = match File::open(&path) {
        Err(e) => panic!("Couldn't open {}: {:?}", display, e),
        Ok(file) => file
    };

    let bz2_reader = BzDecoder::new(file);
    let buf_reader = BufReader::new(bz2_reader);

    let mut total = 0;
    for line in buf_reader.lines() {
        total += line.unwrap().len() + 1;
        // println!("{}", line.unwrap());
    }

    println!("Total: {}", total);
}

/* interpret pages */

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
    let mut inpage = false;
    let mut intitle = false;

    // let mut txt = Vec::new();
    let mut buf = Vec::new();

    // The `Reader` does not implement `Iterator` because it outputs borrowed data (`Cow`s)
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => {
                match e.name() {
                    b"page" => {
                        inpage = true;
                    },
                    b"title" => {
                        intitle = true;
                    }
                    _ => (),
                }
                /*
                let text = String::from_utf8(e.name().to_vec()).unwrap();
                println!("Start element: {}", text);
                */
            },
            Ok(Event::End(ref e)) => {
                match e.name() {
                    b"page" => {
                        inpage = false;
                        total += 1;
                    },
                    b"title" => {
                        intitle = false;
                    }
                    _ => (),
                }
                /*
                let text = String::from_utf8(e.name().to_vec()).unwrap();
                println!("End element: {}", text);
                */
            },
            Ok(Event::Text(e)) => {
                if inpage && intitle {
                    let text = String::from_utf8(e.escaped().to_vec()).unwrap();
                    let pos = reader.buffer_position();
                    println!("{}: {}", pos, text);
                }
            },
            Ok(Event::Eof) => break, // exits the loop when reaching end of file
            Ok(Event::Empty(e)) => {
                /*
                let text = e.unescape_and_decode(&reader).unwrap();
                println!("Empty event: {}", text);
                */
            }
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            e => println!("Unknown event {:?}", e), // There are several other `Event`s we do not consider here
        }

        // if we don't keep a borrow elsewhere, we can clear the buffer to keep memory usage low
        buf.clear();
    }

    println!("Position: {:}", reader.buffer_position());
    println!("{}", total);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let fname_inp = &args[1];
    println!("Parsing {}", fname_inp);
    cat(fname_inp);
}
