use std::time::Duration;
use std::fmt::{Formatter, Result, Display};



pub struct Flags {
    pub merge_rule: MergeRule,
    pub slowing_down: bool,
}

impl Default for Flags {
    fn default() -> Self {
        Flags {
            merge_rule: MergeRule::SimplifiedStandardRules,
            slowing_down: false,
        }
    }
}

pub enum MergeRule {
    SimplifiedStandardRules,

    FastAdvancedRules,
}

pub struct Statistic {
    pub time_for_xml_parsing: Duration,
    pub time_for_io_tree_parsing: Duration,

    pub timestamps_vector: Vec<(Duration,Duration)>,

    pub time_for_decompression: Duration,

    pub time_for_traverse: Duration,
    pub time_for_detraverse: Duration,

    pub number_of_merge_rounds: usize,

    pub number_of_nodes_in_io_tree: usize,
    pub number_of_leafs_in_io_tree: usize,
    pub number_of_edges_in_io_tree: usize,

    pub number_of_leafs_in_top_dag: usize,
    pub number_of_nodes_in_top_dag: usize,
}

impl Statistic {
    pub fn new() -> Statistic {
        Statistic {
            time_for_xml_parsing: Duration::default(),
            time_for_io_tree_parsing: Duration::default(),

            timestamps_vector: Vec::new(),

            time_for_decompression: Duration::default(),

            time_for_traverse: Duration::default(),
            time_for_detraverse: Duration::default(),

            number_of_merge_rounds: 0,

            number_of_nodes_in_io_tree: 0,
            number_of_leafs_in_io_tree: 0,
            number_of_edges_in_io_tree: 0,

            number_of_leafs_in_top_dag: 0,
            number_of_nodes_in_top_dag: 0,
        }
    }
}

impl Display for Statistic {
    fn fmt(&self, f: &mut Formatter) -> Result {
        if Duration::default() != self.time_for_xml_parsing {
            writeln!(f, "Xml parsing toke: {:?}", self.time_for_xml_parsing)?;
        }

        if Duration::default() != self.time_for_io_tree_parsing {
            writeln!(f, "IO tree parsing toke: {:?}", self.time_for_io_tree_parsing)?;
        }

        let mut horizontal_merge_timestamp = Duration::new(0,0);
        let mut vertical_merge_timestamp = Duration::new(0,0);
        for (timestamp_1, timestamp_2) in &self.timestamps_vector {
            horizontal_merge_timestamp += timestamp_1.clone();
            vertical_merge_timestamp += timestamp_2.clone();
            //writeln!(f, "Horizontal merge toke: {:?}, Vertical merge toke: {:?}", timestamp_1, timestamp_2)?;
        }
        writeln!(f, "")?;

        writeln!(f, "Horizontal merges overall toke: {:?}, Vertical merges overall toke: {:?},", horizontal_merge_timestamp, vertical_merge_timestamp)?;
        writeln!(f, "We needed {} rounds", self.number_of_merge_rounds)?;
        writeln!(f, "")?;

        writeln!(f, "Number of nodes in the IO tree: {}", self.number_of_nodes_in_io_tree)?;
        writeln!(f, "Number of leafs in the IO tree: {}", self.number_of_leafs_in_io_tree)?;
        writeln!(f, "Number of edges in the IO tree: {}", self.number_of_edges_in_io_tree)?;

        writeln!(f, "Number of leafs in the TopDAG: {}", self.number_of_leafs_in_top_dag)?;
        writeln!(f, "Number of nodes in the TopDAG: {}", self.number_of_nodes_in_top_dag)
    }
}