#![allow(non_snake_case)]
#![allow(dead_code)]
extern crate xml;

mod structs;

use xml::reader::{EventReader, XmlEvent};

use structs::{Node, Leaf, Edge, ClusterID, InputNode, ParseError};

use std::collections::HashMap;
use std::fs::File;
use std::error::Error;
use std::io::Read;


type GenError = Box<Error>;
type GenResult<T> = Result<T, GenError>;

pub struct TopTreeBuilder {
    label_counter: usize,
    labels: HashMap<String, usize>,
    nodes: Vec<Node>,
    leafs: Vec<Leaf>,
    edges: Vec<Edge>,
    cluster_counter: usize,
    clusters: HashMap<ClusterID, usize>,
}

impl TopTreeBuilder {
    fn new_from_xml(path: &str) -> GenResult<TopTreeBuilder> {
        let mut builder = TopTreeBuilder {
            label_counter: 0,
            labels: HashMap::new(),
            nodes: Vec::new(),
            leafs: Vec::new(),
            edges: Vec::new(),
            cluster_counter: 0,
            clusters: HashMap::new(),
        };

        let file = File::open(path)?;
        let mut reader = EventReader::new(file);

        let root = parse_xml(&mut reader, None, &mut builder)?;


        panic!("");
    }
}

fn parse_xml<B: Read>(mut reader: &mut EventReader<B>, mut node: Option<InputNode>, builder: &mut TopTreeBuilder) -> GenResult<InputNode> {
    'filereader: loop {
        match reader.next()? {
            XmlEvent::EndElement {
                name
            } => {
                if let Some(elem) = node {
                    //insert the label
                    let mut label_id = builder.label_counter;
                    if let Some(old_label) = builder.labels.insert(name.to_string(), label_id) {
                        builder.labels.insert(name.to_string(), old_label);
                        label_id = old_label;
                    } else {
                        builder.label_counter += 1;
                    }

                    //check if label is the same
                    if label_id == elem.label {
                        return Ok(elem);
                    } else {
                        return Err(Box::new(ParseError::CannotParse));
                    }
                }
            },

            XmlEvent::StartElement {
                ref name,
                attributes: _,
                namespace: _,
            } => {
                //insert the label
                let mut label_id = builder.label_counter;
                if let Some(old_label) = builder.labels.insert(name.to_string(), label_id) {
                    builder.labels.insert(name.to_string(), old_label);
                    label_id = old_label;
                } else {
                    builder.label_counter += 1;
                }

                //build new node
                if let Some(ref mut elem) = node {
                    let new_elem = InputNode {
                        label: label_id,
                        children: Vec::new(),
                    };

                    elem.children.push(parse_xml(&mut reader, Some(new_elem), builder)?);
                } else {
                    let root = InputNode {
                        label: label_id,
                        children: Vec::new(),
                    };
                    return parse_xml(&mut reader, Some(root), builder);
                }
            },

            XmlEvent::EndDocument => {
                return Err(Box::new(ParseError::CannotParse));

            },

            XmlEvent::Characters(_)
            | XmlEvent::Whitespace(..)
            | XmlEvent::Comment(..)
            | XmlEvent::CData(_)
            | XmlEvent::StartDocument { .. }
            | XmlEvent::ProcessingInstruction { .. } => { continue },
        }
    }
}

