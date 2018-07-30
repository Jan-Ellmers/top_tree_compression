use std::ops::{Deref, DerefMut};
use std::error::Error;
use std::fmt::{Result, Formatter, Display};

pub struct InputNode {
    pub label: usize,
    pub children: Vec<InputNode>,
}

pub struct Node {
    pub deleted: bool,
    pub label: usize,
    pub first_child: usize,
    pub last_child: usize,
}

pub struct Leaf {
    pub deleted: bool,
    pub label: usize,
}

pub struct Edge {
    pub deleted: bool,
    pub cluster: bool,
    pub index: usize,
}

#[derive(Eq, PartialEq, Hash)]
pub enum ClusterID {
    Leaf {
        ///The label identifier
        label: usize
    },

    Cluster {
        ///The merge type
        merge_type: MergeType,
        ///The cluster identifier
        first_child: usize,
        ///The cluster identifier
        second_child: usize,
    },
}

#[derive(Eq, PartialEq, Hash)]
pub enum MergeType {
    A,

    C,

    D,
}

pub struct SaveOption<T> {
    pub value: Option<T>,
}

impl<T> SaveOption<T> {
    pub fn new() -> SaveOption<T> {
        SaveOption {
            value: None,
        }
    }

    pub fn set_value(&mut self, new_value: T) {
        self.value = Some(new_value);
    }

    pub fn swap_value(&mut self, new_value: T) -> T {
        let to_return;
        if let Some(data) = self.value.take() {
            to_return = data;
        } else {
            panic!("Error: Called swap_value on a SaveOption before feeding it any value");
        }
        self.value = Some(new_value);
        to_return
    }

    pub fn is_set(&self) -> bool {
        self.value.is_some()
    }
}

impl<T> Deref for SaveOption<T> {
    type Target = T;

    fn deref(&self) -> &T {
        if let Some(ref data) = self.value {
            data
        } else {
            panic!("Error: Called derefMut on a SaveOption before feeding it any value");
        }
    }
}

impl<T> DerefMut for SaveOption<T> {
    fn deref_mut(&mut self) -> &mut T {
        if let Some(ref mut data) = self.value {
            data
        } else {
            panic!("Error: Called deref_mut on a SaveOption before feeding it any value");
        }
    }
}

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
