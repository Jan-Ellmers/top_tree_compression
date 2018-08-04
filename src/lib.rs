extern crate quick_xml;

use quick_xml::Reader;
use quick_xml::events::Event;


mod structs;

use structs::{Node, Leaf, Edge, ClusterID, ParseError, Child, NodeHandle, MergeType, Data};


use std::collections::HashMap;
use std::fs::File;
use std::error::Error;
use std::io::BufReader;
use std::fmt::{Debug, Formatter, Result, Write, Display};
use std::time::{Instant, Duration};
use std::str::FromStr;


type GenError = Box<Error>;
type GenResult<T> = std::result::Result<T, GenError>;

const DUMMY_NODE_LABEL: &str = "Dummy_node";

pub struct TopTreeBuilder {
    labels: HashMap<String, usize>,
    nodes: Vec<Node>,
    leafs: Vec<Leaf>,
    edges: Vec<Edge>,
    clusters: HashMap<ClusterID, usize>,

    cluster_vector: Vec<(ClusterID, usize)>,
    label_vector: Vec<(String, usize)>,
}

impl TopTreeBuilder {
    pub fn new_from_xml(path: &str) -> GenResult<TopTreeBuilder> {
        let root;
        if cfg!(feature = "performance_test") {
            let time_stamp = Instant::now();
            root = IO_Tree::new_from_xml(path)?;
            println!("Converting the XML to the IO tree toke: {:?}", time_stamp.elapsed());
        } else {
            root = IO_Tree::new_from_xml(path)?;
        }

        Ok(TopTreeBuilder::new_from_IO_tree(root))
    }

    #[allow(non_snake_case)]
    pub fn new_from_IO_tree(tree: IO_Tree) -> TopTreeBuilder {
        let mut builder = TopTreeBuilder {
            labels: HashMap::new(),
            nodes: Vec::with_capacity(4_000_000),
            leafs: Vec::with_capacity(4_000_000),
            edges: Vec::with_capacity(4_000_000),
            clusters: HashMap::new(),

            cluster_vector: Vec::new(),
            label_vector: Vec::new(),
        };

        //insert the dummy label
        let dummy_label_id = builder.insert_label(&DUMMY_NODE_LABEL.to_owned());
        builder.nodes.push(Node::new(dummy_label_id));

        //insert a place holder child
        builder.edges.push(Edge::default());

        //insert the root
        let root_label_id = builder.insert_label(&tree.label);
        let child = if tree.children.len() == 0 {
            Child::Leaf(Leaf {deleted: false, data: Data::Label(root_label_id)})
        } else {
            Child::Node(Node::new(root_label_id), Some(tree.children.len()))
        };
        let root_addr = builder.push_child(0, child);

        //insert the tree
        if cfg!(feature = "performance_test") {
            let time_stamp = Instant::now();
                builder.rec_insert_tree(root_addr, tree);
            println!("Parsing the IO tree toke: {:?}", time_stamp.elapsed());
        } else {
            builder.rec_insert_tree(root_addr, tree);
        }

        //build the TopDag
        let mut step_1_time = Duration::new(0, 0);
        let mut step_2_time = Duration::new(0, 0);

        let mut number_of_steps = 0;

        while !builder.finished() {
            if cfg!(feature = "performance_test") {
                number_of_steps += 1;
                println!("\nNew Round:");
                let mut time_stamp = Instant::now();

                builder.step_1();

                let first_timestamp = time_stamp.elapsed();
                if cfg!(feature = "debug") {
                    println!("Step 1 finished");
                    println!("{:?}", builder);
                }
                time_stamp = Instant::now();

                builder.step_2();

                let second_timestamp = time_stamp.elapsed();
                if cfg!(feature = "debug") {
                    println!("Step 2 finished");
                    println!("{:?}", builder);
                }
                step_1_time += first_timestamp;
                step_2_time += second_timestamp;
                println!("\nStep 1 toke: {:?}, Step 2 toke: {:?}", first_timestamp, second_timestamp);
            } else {
                builder.step_1();
                if cfg!(feature = "debug") {
                    println!("Step 1 finished");
                    println!("{:?}", builder);
                }
                builder.step_2();
                if cfg!(feature = "debug") {
                    println!("Step 2 finished");
                    println!("{:?}", builder);
                }
            }
        }
        if cfg!(feature = "performance_test") {
            println!("\nStep 1 overall toke: {:?}, Step 2 overall toke: {:?}", step_1_time, step_2_time);
            println!("\nBuilding the DAG overall toke {:?} and {} rounds", step_1_time + step_2_time, number_of_steps);
        }
        builder
    }

    #[allow(non_snake_case)]
    pub fn into_IO_tree(self) -> IO_Tree {
        let mut dummy_node = IO_Tree {
            label: DUMMY_NODE_LABEL.to_owned(),
            children: Vec::new(),
        };

        dummy_node.children.push(IO_Tree {
            label: (self.cluster_vector.len() - 1).to_string(),
            children: Vec::new(),
        });

        self.rec_into_IO_tree(&mut dummy_node, 0);

        assert!(dummy_node.children.len() == 1);
        dummy_node.children.pop().unwrap()
    }

    #[allow(non_snake_case)]
    fn rec_into_IO_tree(&self, parent: &mut IO_Tree, index: usize) {
        if let Ok(cluster_index) = usize::from_str(&parent.children[index].label) {
            let (cluster, _index) = &self.cluster_vector[cluster_index];
            assert!(*_index == cluster_index);

            match cluster {
                ClusterID::Cluster {merge_type: MergeType::AB, first_child, second_child} => {
                    //we add the first cluster over the second. The existing child transforms to the second cluster

                    let new_node = IO_Tree {
                        label: first_child.to_string(),
                        children: Vec::new(),
                    };

                    //swap child and new_node
                    parent.children.push(new_node);
                    let mut second_cluster = parent.children.swap_remove(index);

                    //change the label of the old child
                    second_cluster.label = second_child.to_string();

                    //push the child back into the tree
                    parent.children[index].children.push(second_cluster);

                    //rec call on both nodes
                    self.rec_into_IO_tree(&mut parent.children[index], 0);

                    self.rec_into_IO_tree(parent, index);
                },

                ClusterID::Cluster {merge_type: MergeType::C, first_child, second_child} => {
                    parent.children[index].label = first_child.to_string();

                    parent.children.insert(index + 1, IO_Tree {
                        label: second_child.to_string(),
                        children: Vec::new(),
                    });

                    //rec call on both nodes
                    self.rec_into_IO_tree(parent, index + 1);

                    self.rec_into_IO_tree(parent, index);
                },

                ClusterID::Cluster {merge_type: MergeType::DE, first_child, second_child} => {
                    parent.children[index].label = second_child.to_string();

                    parent.children.insert(index, IO_Tree {
                        label: first_child.to_string(),
                        children: Vec::new(),
                    });

                    //rec call on both nodes
                    self.rec_into_IO_tree(parent, index + 1);

                    self.rec_into_IO_tree(parent, index);
                },

                ClusterID::Leaf {label} => {
                    let (ref label, _) = self.label_vector[*label];
                    parent.children[index].label = label.to_owned();
                },
            }
        } else {panic!("Error: Database is corrupt")}
    }

    /// computes if we have finished the TopDag building
    fn finished(&self) -> bool {
        //if the dummy node has only one child and that child is a leaf
        //but dummy node always must have one child
        self.edges[self.nodes[0].first_child].index >= usize::max_value() >> 1
    }

    fn step_1(&mut self) {
        let mut index = 0;
        while index < self.nodes.len() {
            //check if node is not deleted
            if !self.nodes[index].deleted {
                self.step_1_subroutine(index);
            }
            index += 1;
        }
    }

    fn step_1_subroutine(&mut self, parent: usize) {
        //assert all nodes between first and last child are not deleted
        let mut index = self.nodes[parent].first_child;
        while index + 1 < self.nodes[parent].last_child {
            let first_is_leaf = self.edges[index].index >= usize::max_value() >> 1;
            let second_is_leaf = self.edges[index + 1].index >= usize::max_value() >> 1;

            let first_cluster = NodeHandle { parent, child: index };
            let second_cluster = NodeHandle { parent, child: index + 1 };

            if first_is_leaf { //merge type D or E
                self.merge(first_cluster, second_cluster, MergeType::DE);
            } else if second_is_leaf { //merge type C
                self.merge(first_cluster, second_cluster, MergeType::C);
            } //else no merge possible

            index +=2;
        }
        //restore the assertion
        self.compress_children(parent);
    }

    fn step_2(&mut self) {
        let mut index = 0;
        while index < self.nodes.len() {
            //check if node is not deleted
            if !self.nodes[index].deleted {
                if self.nodes[index].first_child + 1 < self.nodes[index].last_child || index == 0 {//node has more than one child or is the dummy node
                    let mut child_index = self.nodes[index].first_child;
                    while child_index < self.nodes[index].last_child {
                        self.step_2_subroutine(NodeHandle { parent: index, child: child_index});
                        child_index += 1;
                    }
                }
            }
            index += 1;
        }
    }

    fn step_2_subroutine(&mut self, mut first_cluster: NodeHandle) {
        loop {
            let second_cluster;
            {//check if second_cluster is valid
                let edge = &mut self.edges[first_cluster.child];
                if edge.index >= usize::max_value() >> 1 { return } //child is a leaf
                let child = &mut self.nodes[edge.index];
                if !(child.first_child + 1 == child.last_child) { return } //child has more than one child

                second_cluster = NodeHandle {
                    parent: edge.index,
                    child: child.first_child
                };
            }

            self.merge(first_cluster.clone(), second_cluster.clone(), MergeType::AB);

            if self.edges[first_cluster.child].index >= usize::max_value() >> 1 { return } //child is a leaf
            first_cluster.parent = self.edges[first_cluster.child].index;
            if !self.nodes[first_cluster.parent].first_child + 1 == self.nodes[first_cluster.parent].last_child { return } //child has more than one child
            first_cluster.child = self.nodes[first_cluster.parent].first_child;
        }
    }

    fn merge(&mut self, first_cluster: NodeHandle, second_cluster: NodeHandle, merge_type: MergeType) {
        use MergeType::{AB,C,DE};
        //get the id of the new cluster
        let cluster_id = ClusterID::Cluster {
            merge_type: merge_type.clone(),
            first_child: self.get_cluster_id_from_child(first_cluster.child),
            second_child: self.get_cluster_id_from_child(second_cluster.child),
        };
        let cluster_id = self.add_cluster(cluster_id);

        match merge_type {
            AB => { //means A or B
                //change the data on the second_cluster_child
                let cluster_child_index = self.edges[second_cluster.child].index;
                if cluster_child_index < usize::max_value() >> 1 {
                    //child is a node so we have a A merge
                    self.nodes[cluster_child_index].data = Data::Cluster(cluster_id);
                } else {
                    //child is a leaf so we have a B merge
                    self.leafs[cluster_child_index - (usize::max_value() >> 1)].data = Data::Cluster(cluster_id);
                }

                //connect first parent mit second child
                self.edges.swap(first_cluster.child, second_cluster.child);

                //delete the unneeded edge and node
                self.edges[second_cluster.child].deleted = true;
                self.nodes[second_cluster.parent].deleted = true;
            },

            C => {
                //change the data on the first_cluster_child
                let cluster_child_index = self.edges[first_cluster.child].index;
                //child is a node because we have a C merge
                self.nodes[cluster_child_index].data = Data::Cluster(cluster_id);

                //delete the unneeded edge and node
                self.edges[second_cluster.child].deleted = true;
                //no check needed we just can delete leafs
                self.leafs[self.edges[second_cluster.child].index - (usize::max_value() >> 1)].deleted = true;
            },

            DE => { //means D or E
                //change the data on the first_cluster_child
                let cluster_child_index = self.edges[second_cluster.child].index;
                if cluster_child_index < usize::max_value() >> 1 {
                    //child is a node so we have a D merge
                    self.nodes[cluster_child_index].data = Data::Cluster(cluster_id);
                } else {
                    //child is a leaf so we have a E merge
                    self.leafs[cluster_child_index - (usize::max_value() >> 1)].data = Data::Cluster(cluster_id);
                }

                //delete the unneeded edge and node
                self.edges[first_cluster.child].deleted = true;
                //no check needed we just can delete leafs
                self.leafs[self.edges[first_cluster.child].index - (usize::max_value() >> 1)].deleted = true;
            },
        }
    }

    /// builds a cluster from the child
    /// child must be an index from the edge array
    /// returns the usize identifier from the child
    fn get_cluster_id_from_child(&mut self, child: usize) -> usize {
        let node_index = self.edges[child].index;
        let node_data = if node_index < usize::max_value() >> 1 {
            //child is a node
            self.nodes[node_index].data.clone()
        } else {
            //child is a leaf
            self.leafs[node_index - (usize::max_value() >> 1)].data.clone()
        };

        match node_data {
            Data::Label(id) => { //we have to add the cluster first
                self.add_cluster(ClusterID::Leaf {label: id})
            },
            Data::Cluster(id) => { //we already have a cluster so we just return the id
                id
            },
        }
    }

    /// adds a cluster to the Cluster HashMap returns the key of the new added cluster
    fn add_cluster(&mut self, cluster: ClusterID) -> usize {
        let mut cluster_id = self.cluster_vector.len();
        if let Some(old_cluster_id) = self.clusters.insert(cluster.clone(), cluster_id) {
            self.clusters.insert(cluster.clone(), old_cluster_id);
            cluster_id = old_cluster_id;
        } else { //cluster was not inserted jet
            self.cluster_vector.push((cluster, cluster_id));
        }
        cluster_id
    }

    fn rec_insert_tree(&mut self, node: usize, tree: IO_Tree) {
        for child in tree.children {
            //insert label
            if tree.label == DUMMY_NODE_LABEL { panic!("Error: Node must not be called {}", DUMMY_NODE_LABEL) }
            let label_id = self.insert_label(&child.label);

            let number_of_children = child.children.len();
            let new_child = if number_of_children == 0 {
                Child::Leaf(Leaf{ deleted: false, data: Data::Label(label_id) })
            } else {
                Child::Node(Node::new(label_id), Some(number_of_children))
            };

            let pos = self.push_child(node, new_child);
            self.rec_insert_tree(pos, child);
        }
    }

    fn insert_label(&mut self, name: &String) -> usize {
        let mut label_id = self.label_vector.len();
        if let Some(old_label) = self.labels.insert(name.clone(), label_id) {
            self.labels.insert(name.clone(), old_label);
            label_id = old_label;
        } else { //label was not inserted jet
            self.label_vector.push((name.to_string(), label_id));
        }
        label_id
    }

    ///returns the position of the child in the node- or leaf array
    fn push_child(&mut self, parent: usize, child: Child) -> usize {
        use structs::Child::{Node, Leaf};
        if parent < usize::max_value() >> 1 {
            let last_child = self.nodes[parent].last_child;

            //add the child to the Vector
            let child_addr;
            match child {
                Leaf(leaf) => {
                    child_addr = self.leafs.len() + (usize::max_value() >> 1);
                    self.leafs.push(leaf);
                },

                Node(node, number_of_children) => {
                    let number_of_children = number_of_children.unwrap_or(1);
                    //get the new addr from child an push it to that addr
                    child_addr = self.nodes.len();
                    if child_addr >= usize::max_value() >> 1 {panic!("Error: To many nodes");}
                    self.nodes.push(node);

                    //set the child addr
                    self.nodes[child_addr].first_child = self.edges.len();
                    self.nodes[child_addr].last_child = self.edges.len();

                    //reserve space for future children
                    for _ in 0..number_of_children {
                        self.edges.push(Edge::default());
                    }
                },
            }

            //add the addr to child
            //if     last child is not greater than the array
            //   and the edge at that position is deleted so we can overwrite it
            //   and we do not interfere with the next node first_child pointer
            if     last_child < self.edges.len() 
                && self.edges[last_child].deleted 
                && (( parent + 1 < self.nodes.len() 
                      && self.nodes[parent + 1].first_child > last_child)
                      || parent + 1 >= self.nodes.len()) {

                //replace edge
                self.edges[last_child].deleted = false;
                self.edges[last_child].index = child_addr;

                //adjust last child addr
                self.nodes[parent].last_child += 1;
            } else {
                //insert child
                self.edges.insert(last_child, Edge {
                    deleted: false,
                    index: child_addr});

                //adjust last child addr
                self.nodes[parent].last_child += 1;

                //check if we are at the end of the edges array
                if last_child < self.edges.len() {
                    println!("Warning: Edge buffer is empty");

                    //adjust all node child addr
                    let mut index = parent + 1;
                    while index < self.nodes.len() {
                        self.nodes[index].first_child += 1;
                        self.nodes[index].last_child += 1;
                        index += 1;
                    }
                }
            }

            child_addr
        } else {
            panic!("Error: Can not push a Child to a Leaf");
        }
    }

    ///swaps the children around to restore the assertion from the Node struct
    fn compress_children(&mut self, parent: usize) {
        //find first not deleted edge
        let mut backward_index = self.nodes[parent].first_child;
        while !(backward_index >= self.nodes[parent].last_child || self.edges[backward_index].deleted) {
            backward_index += 1;
        }
        let mut forward_index = backward_index + 1;

        //swap nodes
        while forward_index < self.nodes[parent].last_child {
            if !self.edges[forward_index].deleted { //node is not deleted
                self.edges.swap(backward_index, forward_index);
                backward_index += 1;
                forward_index += 1;
            } else { //node is deleted
                forward_index += 1;
            }
        }

        //adjust last child
        self.nodes[parent].last_child = backward_index;
    }
}

impl Debug for TopTreeBuilder {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let mut output = String::new();
        write!(output, "Label HashMap: \n")?;
        let mut labels = vec![];
        for label in self.labels.clone() {
            labels.push(label);
        }
        labels.sort_by_key(|value| {value.1});
        for label in labels {
            write!(output, "{:?} \n", label)?;
        }

        write!(output, "\nNodes Vector: \n")?;
        for node in self.nodes.clone() {
            write!(output, "{:?} \n", node)?;
        }

        write!(output, "\nLeaf Vector: \n")?;
        for leaf in self.leafs.clone() {
            write!(output, "{:?} \n", leaf)?;
        }

        write!(output, "\nEdge Vector: \n")?;
        for mut edge in self.edges.clone() {
            if edge.index < usize::max_value() >> 1 { //node
                write!(output, "Node: ")?;
            } else { //leaf
                write!(output, "Leaf: ")?;
            }
            edge.index %= usize::max_value() >> 1;
            write!(output, "{:?} \n", edge)?;
        }

        write!(output, "\nCluster HashMap: \n")?;
        let mut clusters = vec![];
        for cluster in self.clusters.clone() {
            clusters.push(cluster);
        }
        clusters.sort_by_key(|value| {value.1});
        for cluster in clusters {
            write!(output, "{:?} \n", cluster)?;
        }

        write!(f, "Debug info: \n{}", output)
    }
}

impl Display for TopTreeBuilder {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let mut output = String::new();

        writeln!(output, "Number of unique labels {}", self.labels.len() - 1)?;

        writeln!(output, "Number of nodes in the IO Tree {}", self.nodes.len() - 1)?;

        writeln!(output, "Number of Leafs in the IO Tree {}", self.leafs.len())?;

        writeln!(output, "Number of Edges in the IO Tree {}", self.edges.len() - 1)?;

        let mut number_of_leafs = 0;
        let mut number_of_nodes = 0;
        for (cluster_id, _number) in self.clusters.clone() {
            match cluster_id {
                ClusterID::Cluster {..} => {number_of_nodes += 1},
                ClusterID::Leaf {..} => {number_of_leafs += 1},
            }
        }

        writeln!(output, "Number of leafs in the TopDAG {}", number_of_leafs)?;
        writeln!(output, "Number of nodes in the TopDAG {}", number_of_nodes)?;

        write!(f,"{}", output)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(non_camel_case_types)]
pub struct IO_Tree {
    pub label: String,
    pub children: Vec<IO_Tree>,
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
                        children: Vec::new(),
                    });
                },

                Ok(Event::End(ref elem)) => {
                    let label = String::from_utf8_lossy(elem.name()).to_string();
                    if let Some(node) = node_stack.pop() {
                        if node.label != label {return Err(Box::new(ParseError::CannotParse));}
                        if let Some(last) = node_stack.len().checked_sub(1) {
                            //node is not root so we push it to its parent
                            node_stack[last].children.push(node);
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
