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