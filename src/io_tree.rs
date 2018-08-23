use std::collections::VecDeque;
use std::fs::File;
use std::error::Error;
use std::io::BufReader;
use std::fmt::{Formatter, Result, Display};

use quick_xml::Reader;
use quick_xml::events::Event;


pub type GenError = Box<Error>;
pub type GenResult<T> = std::result::Result<T, GenError>;



#[derive(Debug)]
pub enum ParseError {
    CannotParse,
}

impl Error for ParseError {
    fn description(&self) -> &str {
        "Xml parse error"
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "Cannot Parse")
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(non_camel_case_types)]
pub struct IO_Tree {
    pub label: String,
    pub children: VecDeque<IO_Tree>,
}

impl IO_Tree {
    pub fn new_from_xml (path: &str) -> GenResult<IO_Tree> {
        let file = File::open(path)?;
        let mut reader = Reader::from_reader(BufReader::new(file));

        let mut buf = Vec::new();

        let mut node_stack = Vec::new();

        'filereader: loop {
            match reader.read_event(&mut buf) {
                Ok(Event::Start(ref elem)) => {
                    let label = String::from_utf8_lossy(elem.name()).to_string();

                    node_stack.push(IO_Tree {
                        label,
                        children: VecDeque::new(),
                    });
                },

                Ok(Event::End(ref elem)) => {
                    let label = String::from_utf8_lossy(elem.name()).to_string();
                    if let Some(node) = node_stack.pop() {
                        if node.label != label {return Err(Box::new(ParseError::CannotParse));}
                        if let Some(last) = node_stack.len().checked_sub(1) {
                            //node is not root so we push it to its parent
                            node_stack[last].children.push_back(node);
                        } else { //push the root back on the stack
                            node_stack.push(node);
                        }
                    } else {
                        return Err(Box::new(ParseError::CannotParse));
                    }
                },

                Ok(Event::Eof) => break 'filereader, // exits the loop when reaching end of file

                Err(_) => {
                    return Err(Box::new(ParseError::CannotParse))
                },

                _ => (), // There are several other `Event`s we do not consider here
            }

            // if we don't keep a borrow elsewhere, we can clear the buffer to keep memory usage low
            buf.clear();
        }

        if let Some(root) = node_stack.pop() {
            if node_stack.is_empty() {
                Ok(root)
            } else {
                Err(Box::new(ParseError::CannotParse))
            }
        } else {
            Err(Box::new(ParseError::CannotParse))
        }
    }
}
