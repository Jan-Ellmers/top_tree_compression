use std::ops::{Deref, DerefMut};
use std::error::Error;
use std::fmt::{Result, Formatter, Display};

#[derive(Clone, Debug)]
pub struct NodeHandle {
    ///parents pos in the node vector
    pub parent: usize,
    ///pos of the child in the edge vector
    pub child: usize,
}

#[derive(Clone, Debug)]
pub struct Leaf {
    pub deleted: bool,
    pub data: Data,
}

pub enum Child {
    ///A node and the option to set a request for number of children that will be added
    Node(Node, Option<usize>),
    ///A Leaf
    Leaf(Leaf),
}

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct Cluster {
    ///The merge type
    pub merge_type: MergeType,
    ///The cluster identifier
    pub first_child: usize,
    ///The cluster identifier
    pub second_child: usize,
}

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub enum MergeType {
    AB,

    C,

    DE,
}

impl MergeType {
    pub fn from_usize(integer: usize) -> Self {
        match integer {
            0 => MergeType::AB,
            1 => MergeType::C,
            _ => MergeType::DE,
        }
    }

    pub fn get_usize(&self) -> usize {
        use MergeType::{AB,C,DE};
        match self {
            AB => 0,
            C => 1,
            DE => 2,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Data {
    Label(usize),
    Cluster(usize),
}



#[derive(Clone, Debug)]
/// A Node in the Input Tree
/// assert all children between first and last child are not deleted
pub struct Node {
    pub deleted: bool,
    pub data: Data,
    ///The position of the first child in the edges array (inclusive)
    pub first_child: usize,
    ///The position of the last child in the edges array (exclusive)
    pub last_child: usize,
}

impl Node {
    pub fn new(label: usize) -> Node {
        Node {
            deleted: false,
            data: Data::Label(label),
            first_child: 0,
            last_child: 0,
        }
    }
}



#[derive(Clone, Debug)]
pub struct Edge {
    pub deleted: bool,
    pub index: usize,
}

impl Default for Edge {
    fn default() -> Edge {
        Edge { deleted: true, index: 0 }
    }
}


#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Uninitialized<T> {
    pub value: Option<T>,
}

impl<T> Uninitialized<T> {
    pub fn new() -> Uninitialized<T> {
        Uninitialized {
            value: None,
        }
    }

    pub fn into_inner(mut self) -> T {
        self.value.take().unwrap()
    }

    pub fn try_into_inner(mut self) -> Option<T> {
        self.value.take()
    }

    pub fn set_value(&mut self, new_value: T) {
        self.value = Some(new_value);
    }

    /*pub fn swap_value(&mut self, new_value: T) -> T {
        let to_return;
        if let Some(data) = self.value.take() {
            to_return = data;
        } else {
            panic!("Error: Called swap_value on a SaveOption before feeding it any value");
        }
        self.value = Some(new_value);
        to_return
    }*/

    pub fn is_initialized(&self) -> bool {
        self.value.is_some()
    }

    pub fn is_uninitialized(&self) -> bool {
        self.value.is_none()
    }
}

impl<T> Deref for Uninitialized<T> {
    type Target = T;

    fn deref(&self) -> &T {
        if let Some(ref data) = self.value {
            data
        } else {
            panic!("Error: Use of a not initialized value");
        }
    }
}

impl<T> DerefMut for Uninitialized<T> {
    fn deref_mut(&mut self) -> &mut T {
        if let Some(ref mut data) = self.value {
            data
        } else {
            panic!("Error: Use of a not initialized value");
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
