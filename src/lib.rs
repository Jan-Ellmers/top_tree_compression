#![feature(extern_prelude)]
extern crate quick_xml;

#[macro_use]
mod macros;
mod structs;
pub mod io_tree;
pub mod flags_and_statistic;
mod uninitialized;

use structs::{Node, Leaf, Edge, Cluster, Child, NodeHandle, MergeType, Data};
use io_tree::{IO_Tree, GenResult};
use flags_and_statistic::{Statistic, Flags, MergeRule};
use uninitialized::Uninitialized;


use std::collections::{HashMap, VecDeque};
use std::fmt::{Debug, Formatter, Result, Write};
use std::time::Instant;
use std::str::FromStr;



const DUMMY_NODE_LABEL: &str = "Dummy_node";

pub struct TopTreeBuilder {
    nodes: Vec<Node>,
    leafs: Vec<Leaf>,
    edges: Vec<Edge>,


    clusters: HashMap<Cluster, usize>,
    labels: HashMap<String, usize>,

    cluster_vector: Vec<Cluster>,
    cluster_size: Vec<usize>,
    label_vector: Vec<String>,

    flags: Flags,
    statistic: Statistic,

    number_of_steps: usize,
}

impl TopTreeBuilder {
    pub fn new_from_xml(path: &str, flags: Option<Flags>) -> GenResult<TopTreeBuilder> {
        let mut builder = TopTreeBuilder {
            nodes: Vec::with_capacity(40_000_000),
            leafs: Vec::with_capacity(40_000_000),
            edges: Vec::with_capacity(40_000_000),


            clusters: HashMap::new(),
            labels: HashMap::new(),

            cluster_vector: Vec::new(),
            cluster_size: Vec::new(),
            label_vector: Vec::new(),

            flags: flags.unwrap_or_default(),
            statistic: Statistic::new(),

            number_of_steps: 0,
        };

        let root = measure_performance!(IO_Tree::new_from_xml(path)?, builder.statistic.time_for_xml_parsing);

        builder.build_from_IO_tree(root);

        Ok(builder)
    }

    #[allow(non_snake_case)]
    pub fn new_from_IO_tree (tree: IO_Tree, flags: Option<Flags>) -> TopTreeBuilder {
        let mut builder = TopTreeBuilder {
            nodes: Vec::with_capacity(40_000_000),
            leafs: Vec::with_capacity(40_000_000),
            edges: Vec::with_capacity(40_000_000),


            clusters: HashMap::new(),
            labels: HashMap::new(),

            cluster_vector: Vec::new(),
            cluster_size: Vec::new(),
            label_vector: Vec::new(),

            flags: flags.unwrap_or_default(),
            statistic: Statistic::new(),

            number_of_steps: 0,
        };

        builder.build_from_IO_tree(tree);

        builder
    }

    #[allow(non_snake_case)]
    pub fn build_from_IO_tree(&mut self, tree: IO_Tree) {
        //insert the dummy label
        let dummy_label_id = self.insert_label(&DUMMY_NODE_LABEL.to_owned());
        self.nodes.push(Node::new(dummy_label_id));

        //insert a place holder child
        self.edges.push(Edge::default());

        //insert the root
        let root_label_id = self.insert_label(&tree.label);
        let child = if tree.children.len() == 0 {
            Child::Leaf(Leaf {deleted: false, data: Data::Label(root_label_id)})
        } else {
            Child::Node(Node::new(root_label_id), Some(tree.children.len()))
        };
        let root_addr = self.push_child(0, child);

        //insert the tree
        measure_performance!(self.rec_insert_tree(root_addr, tree), self.statistic.time_for_io_tree_parsing);

        //build the TopDag
        //stop if dummy has only a leaf as child
        while self.edges[self.nodes[0].first_child].index < usize::max_value() >> 1 {
            if cfg!(feature = "performance_test") {
                self.number_of_steps += 1;
                let mut time_stamp = Instant::now();

                self.horizontal_merge();

                let first_timestamp = time_stamp.elapsed();
                debug!("Horizontal merge finished\n{:?}", self);
                time_stamp = Instant::now();

                self.vertical_merge();

                let second_timestamp = time_stamp.elapsed();
                debug!("Vertical merge finished\n{:?}", self);
                self.statistic.timestamps_vector.push((first_timestamp, second_timestamp));
            } else {
                self.number_of_steps += 1;
                self.horizontal_merge();
                debug!("Horizontal merge finished\n{:?}", self);
                self.vertical_merge();
                debug!("Vertical merge finished\n{:?}", self);
            }
        }
        //make statistic
        self.statistic.number_of_merge_rounds = self.number_of_steps;
        self.statistic.number_of_nodes_in_io_tree = self.nodes.len();
        self.statistic.number_of_leafs_in_io_tree = self.leafs.len();
        self.statistic.number_of_edges_in_io_tree = self.edges.len();

        self.statistic.number_of_leafs_in_top_dag = self.label_vector.len() - 1; //exclude the dummy lable
        self.statistic.number_of_nodes_in_top_dag = self.cluster_vector.len();

        //clear the unneeded vectors
        self.nodes.clear();
        self.leafs.clear();
        self.edges.clear();
    }

    #[allow(non_snake_case)]
    pub fn get_IO_tree(&mut self) -> IO_Tree {
        let mut dummy_node = IO_Tree {
            label: DUMMY_NODE_LABEL.to_owned(),
            children: VecDeque::new(),
        };

        dummy_node.children.push_back(IO_Tree {
            label: (self.cluster_vector.len() - 1 + self.label_vector.len()).to_string(),
            children: VecDeque::new(),
        });

        measure_performance!(self.rec_get_IO_tree(&mut dummy_node, 0), self.statistic.time_for_decompression);

        assert!(dummy_node.children.len() == 1);
        dummy_node.children.pop_back().unwrap()
    }

    pub fn get_statistic(&self) -> &Statistic {
        &self.statistic
    }

    #[allow(non_snake_case)]
    fn rec_get_IO_tree(&self, parent: &mut IO_Tree, index: usize) {
        if let Ok(mut cluster_index) = usize::from_str(&parent.children[index].label) {
            if cluster_index < self.label_vector.len() {
                //we have a leaf
                let label = &self.label_vector[cluster_index];
                parent.children[index].label = label.to_owned();
            } else {
                cluster_index -= self.label_vector.len();
                let cluster = &self.cluster_vector[cluster_index];

                match cluster {
                    Cluster {merge_type: MergeType::AB, first_child, second_child} => {
                        //we add the first cluster over the second. The existing child transforms to the second cluster

                        let new_node = IO_Tree {
                            label: first_child.to_string(),
                            children: VecDeque::new(),
                        };

                        //swap child and new_node
                        parent.children.push_back(new_node);
                        let mut second_cluster = parent.children.swap_remove_back(index).unwrap();

                        //change the label of the old child
                        second_cluster.label = second_child.to_string();

                        //push the child back into the tree
                        parent.children[index].children.push_back(second_cluster);

                        //rec call on both nodes
                        self.rec_get_IO_tree(&mut parent.children[index], 0);

                        self.rec_get_IO_tree(parent, index);
                    },

                    Cluster {merge_type: MergeType::CE, first_child, second_child} => {
                        parent.children[index].label = first_child.to_string();

                        parent.children.insert(index + 1, IO_Tree {
                            label: second_child.to_string(),
                            children: VecDeque::new(),
                        });

                        //rec call on both nodes
                        self.rec_get_IO_tree(parent, index + 1);

                        self.rec_get_IO_tree(parent, index);
                    },

                    Cluster {merge_type: MergeType::DE, first_child, second_child} => {
                        parent.children[index].label = second_child.to_string();

                        parent.children.insert(index, IO_Tree {
                            label: first_child.to_string(),
                            children: VecDeque::new(),
                        });

                        //rec call on both nodes
                        self.rec_get_IO_tree(parent, index + 1);

                        self.rec_get_IO_tree(parent, index);
                    },
                }
            }


        } else {panic!("Error: Database is corrupt")}
    }

    fn horizontal_merge(&mut self) {
        use MergeRule::{SimplifiedStandardRules, FastAdvancedRules, SlowAdvancedRules};
        let mut index = 0;
        while index < self.nodes.len() {
            //check if node is not deleted
            if !self.nodes[index].deleted {
                match self.flags.merge_rule {
                    SimplifiedStandardRules => {
                        self.ssr_horizontal_merge(index);
                    },

                    FastAdvancedRules => {
                        self.far_horizontal_merge(index);
                    }

                    SlowAdvancedRules => {
                        self.sar_horizontal_merge(index);
                    }
                }
            }
            index += 1;
        }
    }

    ///simplified_standard_rules
    fn ssr_horizontal_merge(&mut self, parent: usize) {
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
                self.merge(first_cluster, second_cluster, MergeType::CE);
            } //else no merge possible

            index +=2;
        }
        //restore the assertion
        self.compress_children(parent);
    }

    ///fast_advanced_rules
    fn far_horizontal_merge(&mut self, parent: usize) {
        //assert all nodes between first and last child are not deleted
        let mut index = self.nodes[parent].first_child;

        loop {
            //check first two clusters
            if index + 1 >= self.nodes[parent].last_child {break}
            let first_cluster = NodeHandle { parent, child: index };
            let second_cluster = NodeHandle { parent, child: index + 1 };

            let could_merge;
            if self.edges[index].index >= usize::max_value() >> 1 {
                //type DE
                could_merge = Some(MergeType::DE);
                if self.try_merge(first_cluster.clone(), second_cluster.clone(), MergeType::DE) {
                    self.merge(first_cluster, second_cluster, MergeType::DE);
                    index += 2;
                    continue;
                }

                /*if self.edges[index + 1].index >= usize::max_value() >> 1 { //had bad results in practice
                    //type E so we try to merge it with CE
                    if self.try_merge(first_cluster.clone(), second_cluster.clone(), MergeType::CE) {
                        self.merge(first_cluster, second_cluster, MergeType::CE);
                        index += 2;
                        continue;
                    }
                }*/
            } else if self.edges[index + 1].index >= usize::max_value() >> 1 {
                //type C
                could_merge = Some(MergeType::CE);
                if self.try_merge(first_cluster.clone(), second_cluster.clone(), MergeType::CE) {
                    self.merge(first_cluster, second_cluster, MergeType::CE);
                    index += 2;
                    continue;
                }
            } else {
                could_merge = None;
            }

            //check if we have a thrid cluster
            if index + 2 >= self.nodes[parent].last_child {
                if let Some(merge_type) = could_merge {
                    self.merge(first_cluster, second_cluster, merge_type);
                }
                break;
            }
            let third_cluster = NodeHandle { parent, child: index + 2 };

            //check last two clusters
            if self.edges[index + 1].index >= usize::max_value() >> 1 {
                //type DE
                if self.try_merge(second_cluster.clone(), third_cluster.clone(), MergeType::DE) {
                    self.merge(second_cluster, third_cluster, MergeType::DE);
                    index += 3;
                    continue;
                }
            } else if self.edges[index + 2].index >= usize::max_value() >> 1 {
                //type C
                if self.try_merge(second_cluster.clone(), third_cluster.clone(), MergeType::CE) {
                    self.merge(second_cluster, third_cluster, MergeType::CE);
                    index += 3;
                    continue;
                }
            }

            //no merge with thrid cluster so merge first two if possible
            if let Some(merge_type) = could_merge {
                self.merge(first_cluster, second_cluster, merge_type);
            }
            index += 2;
        }
        //restore the assertion
        self.compress_children(parent);
    }

    fn sar_horizontal_merge(&mut self, parent: usize) {
        if self.number_of_steps % 5 == 0 {
            self.far_horizontal_merge(parent);
        } else {
            //assert all nodes between first and last child are not deleted
            let mut index = self.nodes[parent].first_child;

            loop {
                //check first two clusters
                if index + 1 >= self.nodes[parent].last_child {break}
                let first_cluster = NodeHandle { parent, child: index };
                let second_cluster = NodeHandle { parent, child: index + 1 };

                if self.edges[index].index >= usize::max_value() >> 1 {
                    //type DE
                    if self.try_merge(first_cluster.clone(), second_cluster.clone(), MergeType::DE) {
                        self.merge(first_cluster, second_cluster, MergeType::DE);
                        index += 2;
                        continue;
                    }
                } else if self.edges[index + 1].index >= usize::max_value() >> 1 {
                    //type C
                    if self.try_merge(first_cluster.clone(), second_cluster.clone(), MergeType::CE) {
                        self.merge(first_cluster, second_cluster, MergeType::CE);
                        index += 2;
                        continue;
                    }
                }

                //check if we have a thrid cluster
                if index + 2 >= self.nodes[parent].last_child {
                    break;
                }
                let third_cluster = NodeHandle { parent, child: index + 2 };

                //check last two clusters
                if self.edges[index + 1].index >= usize::max_value() >> 1 {
                    //type DE
                    if self.try_merge(second_cluster.clone(), third_cluster.clone(), MergeType::DE) {
                        self.merge(second_cluster, third_cluster, MergeType::DE);
                        index += 3;
                        continue;
                    }
                } else if self.edges[index + 2].index >= usize::max_value() >> 1 {
                    //type C
                    if self.try_merge(second_cluster.clone(), third_cluster.clone(), MergeType::CE) {
                        self.merge(second_cluster, third_cluster, MergeType::CE);
                        index += 3;
                        continue;
                    }
                }

                //no merge with thrid cluster so merge first two if possible
                index += 2;
            }
            //restore the assertion
            self.compress_children(parent);
        }
    }

    fn vertical_merge(&mut self) {
        use MergeRule::{SimplifiedStandardRules, FastAdvancedRules, SlowAdvancedRules};
        let mut index = 0;
        while index < self.nodes.len() {
            //check if node is not deleted
            if !self.nodes[index].deleted {
                if self.nodes[index].first_child + 1 < self.nodes[index].last_child || index == 0 {//node has more than one child or is the dummy node
                    let mut child_index = self.nodes[index].first_child;
                    while child_index < self.nodes[index].last_child {
                        match self.flags.merge_rule {
                            SimplifiedStandardRules => {
                                self.ssr_vertical_merge(NodeHandle { parent: index, child: child_index});
                            },

                            FastAdvancedRules => {
                                self.far_vertical_merge(NodeHandle { parent: index, child: child_index});
                            }

                            SlowAdvancedRules => {
                                self.sar_vertical_merge(NodeHandle { parent: index, child: child_index});
                            }
                        }

                        child_index += 1;
                    }
                }
            }
            index += 1;
        }
    }

    ///simplified_standard_rules
    fn ssr_vertical_merge(&mut self, mut first_cluster: NodeHandle) {
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

    ///fast_advanced_rules
    fn far_vertical_merge(&mut self, mut first_cluster: NodeHandle) {
        loop {
            let second_parent = self.edges[first_cluster.child].index;
            if second_parent >= usize::max_value() >> 1 { return } //child is a leaf
            if self.nodes[second_parent].first_child + 1 != self.nodes[second_parent].last_child { return } //child has more than one child
            let second_child = self.nodes[second_parent].first_child;
            let second_cluster = NodeHandle { parent: second_parent, child: second_child};

            if self.try_merge(first_cluster.clone(), second_cluster.clone(), MergeType::AB) {
                self.merge(first_cluster.clone(), second_cluster.clone(), MergeType::AB);

                if self.edges[first_cluster.child].index >= usize::max_value() >> 1 { return } //child is a leaf
                first_cluster.parent = self.edges[first_cluster.child].index;
                if self.nodes[first_cluster.parent].first_child + 1 != self.nodes[first_cluster.parent].last_child {return} //child has more than one child
                first_cluster.child = self.nodes[first_cluster.parent].first_child;
                continue
            }

            //build third cluster
            let third_parent = self.edges[second_child].index;
            if third_parent >= usize::max_value() >> 1 {
                self.merge(first_cluster.clone(), second_cluster.clone(), MergeType::AB);
                return
            } //child is a leaf
            if self.nodes[third_parent].first_child + 1 != self.nodes[third_parent].last_child {
                self.merge(first_cluster.clone(), second_cluster.clone(), MergeType::AB);
                return
            } //child has more than one child
            let third_child = self.nodes[third_parent].first_child;
            let third_cluster = NodeHandle { parent: third_parent, child: third_child};

            if self.try_merge(second_cluster.clone(), third_cluster.clone(), MergeType::AB) {
                self.merge(second_cluster.clone(), third_cluster, MergeType::AB);

                if self.edges[second_cluster.child].index >= usize::max_value() >> 1 { return } //child is a leaf
                first_cluster.parent = self.edges[second_cluster.child].index;
                if self.nodes[first_cluster.parent].first_child + 1 != self.nodes[first_cluster.parent].last_child {return} //child has more than one child
                first_cluster.child = self.nodes[first_cluster.parent].first_child;
                continue
            } else {
                self.merge(first_cluster.clone(), second_cluster.clone(), MergeType::AB);

                if self.edges[first_cluster.child].index >= usize::max_value() >> 1 { return } //child is a leaf
                first_cluster.parent = self.edges[first_cluster.child].index;
                if self.nodes[first_cluster.parent].first_child + 1 != self.nodes[first_cluster.parent].last_child {return} //child has more than one child
                first_cluster.child = self.nodes[first_cluster.parent].first_child;
                continue
            }
        }
    }

    fn sar_vertical_merge(&mut self, mut first_cluster: NodeHandle) {
        if self.number_of_steps % 5 == 0 {
            self.far_vertical_merge(first_cluster);
        } else {
            loop {
                let second_parent = self.edges[first_cluster.child].index;
                if second_parent >= usize::max_value() >> 1 { return } //child is a leaf
                if self.nodes[second_parent].first_child + 1 != self.nodes[second_parent].last_child { return } //child has more than one child
                let second_child = self.nodes[second_parent].first_child;
                let second_cluster = NodeHandle { parent: second_parent, child: second_child };

                if self.try_merge(first_cluster.clone(), second_cluster.clone(), MergeType::AB) {
                    self.merge(first_cluster.clone(), second_cluster.clone(), MergeType::AB);

                    if self.edges[first_cluster.child].index >= usize::max_value() >> 1 { return } //child is a leaf
                    first_cluster.parent = self.edges[first_cluster.child].index;
                    if self.nodes[first_cluster.parent].first_child + 1 != self.nodes[first_cluster.parent].last_child { return } //child has more than one child
                    first_cluster.child = self.nodes[first_cluster.parent].first_child;
                    continue
                }

                //build third cluster
                let third_parent = self.edges[second_child].index;
                if third_parent >= usize::max_value() >> 1 {
                    return
                } //child is a leaf
                if self.nodes[third_parent].first_child + 1 != self.nodes[third_parent].last_child {
                    return
                } //child has more than one child
                let third_child = self.nodes[third_parent].first_child;
                let third_cluster = NodeHandle { parent: third_parent, child: third_child };

                if self.try_merge(second_cluster.clone(), third_cluster.clone(), MergeType::AB) {
                    self.merge(second_cluster.clone(), third_cluster, MergeType::AB);

                    if self.edges[second_cluster.child].index >= usize::max_value() >> 1 { return } //child is a leaf
                    first_cluster.parent = self.edges[second_cluster.child].index;
                    if self.nodes[first_cluster.parent].first_child + 1 != self.nodes[first_cluster.parent].last_child { return } //child has more than one child
                    first_cluster.child = self.nodes[first_cluster.parent].first_child;
                    continue
                } else {
                    first_cluster = third_cluster;
                    continue
                }
            }
        }
    }

    ///returns true if the cluster exists already
    fn try_merge(&self, first_cluster: NodeHandle, second_cluster: NodeHandle, merge_type: MergeType) -> bool {
        //get the id of the new cluster
        let first_node = self.edges[first_cluster.child].index;
        let second_node = self.edges[second_cluster.child].index;
        let cluster = Cluster {
            merge_type: merge_type.clone(),
            first_child: self.get_cluster_index(first_node),
            second_child: self.get_cluster_index(second_node),
        };

        self.clusters.get(&cluster).is_some()
    }

    fn merge(&mut self, first_cluster: NodeHandle, second_cluster: NodeHandle, merge_type: MergeType) {
        assert!(first_cluster.parent != second_cluster.parent || merge_type != MergeType::AB);
        use MergeType::{AB,CE,DE};
        //get the id of the new cluster
        let first_node = self.edges[first_cluster.child].index;
        let second_node = self.edges[second_cluster.child].index;
        let cluster = Cluster {
            merge_type: merge_type.clone(),
            first_child: self.get_cluster_index(first_node),
            second_child: self.get_cluster_index(second_node),
        };
        //check for slowing down
        if      cluster.first_child as f64 > self.flags.slowing_down.powf(self.number_of_steps as f64)
             || cluster.second_child as f64 > self.flags.slowing_down.powf(self.number_of_steps as f64) {
            return;
        }

        let cluster_id = self.add_cluster(cluster);

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

            CE => {
                //change the data on the first_cluster_child
                let cluster_child_index = self.edges[first_cluster.child].index;
                if cluster_child_index < usize::max_value() >> 1 {
                    //child is a node so we have a C merge
                    self.nodes[cluster_child_index].data = Data::Cluster(cluster_id);
                } else {
                    //child is a leaf so we have a E merge
                    self.leafs[cluster_child_index - (usize::max_value() >> 1)].data = Data::Cluster(cluster_id);
                }

                //delete the unneeded edge and node
                self.edges[second_cluster.child].deleted = true;
                //no check needed we just can delete leafs
                self.leafs[self.edges[second_cluster.child].index - (usize::max_value() >> 1)].deleted = true;
            },

            DE => { //means D or E
                //change the data on the second_cluster_child
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

    /// builds a cluster from the node
    /// node must be an index from the node array
    /// returns the index of the cluster of the node
    fn get_cluster_index(&self, node: usize) -> usize {
        let node_data = if node < usize::max_value() >> 1 {
            //child is a node
            self.nodes[node].data.clone()
        } else {
            //child is a leaf
            self.leafs[node - (usize::max_value() >> 1)].data.clone()
        };

        match node_data {
            Data::Label(id) => { //we give the id of the lable
                id
            },
            Data::Cluster(id) => { //we already have a cluster so we just return the id
                id
            },
        }
    }

    /// adds a cluster to the Cluster HashMap returns the key of the new added cluster
    fn add_cluster(&mut self, cluster: Cluster) -> usize {
        let mut cluster_id = self.cluster_vector.len() + self.label_vector.len();
        if let Some(old_cluster_id) = self.clusters.insert(cluster.clone(), cluster_id) {
            self.clusters.insert(cluster.clone(), old_cluster_id);
            cluster_id = old_cluster_id;
        } else { //cluster was not inserted jet
            let mut size = 1;
            if cluster.first_child < self.label_vector.len() {
                size += 1;
            } else {
                size += self.cluster_size[cluster.first_child - self.label_vector.len()];
            }

            if cluster.second_child < self.label_vector.len() {
                size += 1;
            } else {
                size += self.cluster_size[cluster.second_child - self.label_vector.len()];
            }

            self.cluster_size.push(size);

            self.cluster_vector.push(cluster);
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
            self.label_vector.push(name.to_string());
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

    pub fn traverse(&self) -> (Vec<bool>, Vec<usize>, Vec<usize>, Vec<String>) { //TODO remove the pub
        let mut structure = Vec::new();
        let mut merge_types = Vec::new();
        let lable = self.label_vector.clone();

        let mut cluster_pointer: Vec<Uninitialized<usize>> = vec![Uninitialized::new(); self.cluster_vector.len()];
        let mut pointer: Vec<Uninitialized<usize>> = vec![Uninitialized::new(); self.cluster_vector.len()*2];

        //(current index, first occurrence)
        let mut work_stack = vec![(self.cluster_vector.len() - 1, true, 0)];
        let mut current_traverse_index: usize = 0;

        //current_index is the true index in the cluster vector
        while let Some((current_index, first_occurrence, own_traverse_index)) = work_stack.pop() {
            let Cluster{ merge_type, first_child, second_child } = &self.cluster_vector[current_index];

            if first_occurrence {
                //increase the traverse index one time for each true cluster
                current_traverse_index += 1;

                //push us on the stack again
                work_stack.push((current_index, false, own_traverse_index));

                //push the merge type
                merge_types.push(merge_type.get_usize());

                //first child
                if *first_child < self.label_vector.len() {
                    //it is a leaf
                    structure.push(false);
                    pointer[own_traverse_index*2].set_value(*first_child);
                } else if cluster_pointer[first_child - self.label_vector.len()].is_initialized() {
                    //we already have this cluster
                    structure.push(false);
                    pointer[own_traverse_index*2].set_value(*cluster_pointer[first_child - self.label_vector.len()]);
                } else {
                    //it is a true cluster
                    structure.push(true);
                    work_stack.push((first_child - self.label_vector.len(), true, current_traverse_index));
                }

                //we need to push a dummy value
                structure.push(false);
            } else {
                //second child
                if *second_child < self.label_vector.len() {
                    //it is a leaf
                    structure[own_traverse_index*2 + 1] = false; //should already be false
                    pointer[own_traverse_index*2 + 1].set_value(*second_child);
                } else if cluster_pointer[second_child - self.label_vector.len()].is_initialized() {
                    //we already have this cluster
                    structure[own_traverse_index*2 + 1] = false; //should already be false
                    pointer[own_traverse_index*2 + 1].set_value(*cluster_pointer[second_child - self.label_vector.len()]);
                } else {
                    //it is a true cluster
                    structure[own_traverse_index*2 + 1] = true;
                    work_stack.push((second_child - self.label_vector.len(), true, current_traverse_index));
                }

                //we have finished this cluster so we mark it as finished
                //we set the pointer to the number of clusters we already have plus the offset for the labels
                cluster_pointer[current_index].set_value(own_traverse_index + self.label_vector.len());
            }
        }
        let pointer = pointer.iter().filter_map(|elem| elem.clone().try_into_inner()).collect();
        (structure, pointer, merge_types, lable)
    }

    pub fn detraverse(&mut self, structure: Vec<bool>, pointer: Vec<usize>, merge_types: Vec<usize>, labels: Vec<String>) { //TODO remove the pub
        //build label Hash Map
        for (index, label) in labels.iter().enumerate() {
            self.labels.insert(label.clone(), index);
        }

        //clear data if we have some
        self.label_vector = labels;
        self.labels.clear();
        self.cluster_vector.clear();
        self.clusters.clear();

        //build index Hash Map
        //the traversal index is not the index in the cluster vector so this maps the traversal index to the cluster index
        let mut traversal_index_to_cluster_index = HashMap::new();

        //build rank Hash Map //TODO maybe replace this by sdsl
        let mut rank = HashMap::new();
        let mut number_of_zeros: usize = 0;
        for (index, bit) in structure.iter().enumerate() {
            if !bit {
                rank.insert(index, number_of_zeros);
                number_of_zeros += 1;
            }
        }

        let mut global_index = 0;
        let mut return_value = 0;
        //(index, merge_type, first_child)
        let mut workstack: Vec<(usize, Uninitialized<MergeType>, Uninitialized<usize>)> = vec![(0, Uninitialized::new(), Uninitialized::new())];


        while let Some((index, mut merge_type, mut first_child)) = workstack.pop() {
            if merge_type.is_uninitialized() {
                //first encounter of this cluster prototype
                merge_type.set_value(MergeType::from_usize(merge_types[index]));
                if structure[index*2] {
                    //push self on stack
                    workstack.push((index, merge_type, first_child));

                    //first child is a true cluster so we push it on the stack
                    global_index += 1;
                    workstack.push((global_index, Uninitialized::new(), Uninitialized::new()));
                } else {
                    return_value = if pointer[*rank.get(&(index*2)).unwrap()] < self.label_vector.len() {
                        //we have a leaf
                        pointer[*rank.get(&(index*2)).unwrap()]
                    } else {
                        //we have a cluster copy
                        *traversal_index_to_cluster_index.get(&(pointer[*rank.get(&(index*2)).unwrap()] - self.label_vector.len())).unwrap()
                    };

                    //push self on stack
                    workstack.push((index, merge_type, first_child));
                    //first child is either a leaf or a copy of a already known cluster so we do not need to push it
                }

            } else if first_child.is_uninitialized() {
                //second encounter of this cluster prototype
                first_child.set_value(return_value);

                if structure[index*2 + 1] {
                    //push self on stack
                    workstack.push((index, merge_type, first_child));

                    //second child is a true cluster so we push it on the stack
                    global_index += 1;
                    workstack.push((global_index, Uninitialized::new(), Uninitialized::new()));
                } else {
                    //second child is either a leaf or a copy of a already known cluster so we do not need to push it
                    let second_child = if pointer[*rank.get(&(index*2 + 1)).unwrap()] < self.label_vector.len() {
                        //we have a leaf
                        pointer[*rank.get(&(index*2 + 1)).unwrap()]
                    } else {
                        //we have a cluster copy
                        *traversal_index_to_cluster_index.get(&(pointer[*rank.get(&(index*2 + 1)).unwrap()] - self.label_vector.len())).unwrap()
                    };
                    //build a new cluster and push it to vector and hash map
                    let cluster = Cluster { merge_type: merge_type.clone().into_inner(), first_child: *first_child, second_child};
                    let cluster_index = self.add_cluster(cluster);
                    traversal_index_to_cluster_index.insert(index, cluster_index);
                    return_value = cluster_index;
                }
            } else {
                //third and last encounter of this cluster prototype
                let second_child = return_value;
                //build a new cluster and push it to vector and hash map
                let cluster = Cluster { merge_type: merge_type.clone().into_inner(), first_child: *first_child, second_child};
                let cluster_index = self.add_cluster(cluster);
                traversal_index_to_cluster_index.insert(index, cluster_index);
                return_value = cluster_index;
            }
        }
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
        for (index, cluster) in clusters.iter().enumerate() {
            write!(output, "{:?} size: {} \n", cluster, self.cluster_size[index])?;
        }

        write!(f, "Debug info: \n{}", output)
    }
}
