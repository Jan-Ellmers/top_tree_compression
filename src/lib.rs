#![allow(non_snake_case)]
#![allow(dead_code)]
use std::collections::HashMap;


pub struct TopTreeBuilder {
    label: HashMap<String, usize>,
    nodes: Vec<Node>,
    leafs: Vec<Leaf>,
    edges: Vec<Edge>,
    clusters: HashMap<ClusterID, usize>,
}

struct Node {
    deleted: bool,
    label: usize,
    first_child: usize,
    last_child: usize,
}

struct Leaf {
    deleted: bool,
    label: usize,
}

struct Edge {
    deleted: bool,
    cluster: bool,
    index: usize,
}

enum ClusterID {
    Leaf{
        label: usize
    },

    Cluster{
        merge_type: MergeType,
        first_child: usize,
        second_child: usize,
    },
}

enum MergeType {
    A,

    C,

    D,
}