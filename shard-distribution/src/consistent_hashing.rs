use crate::hash::hash;
use crate::shard_distribution::ShardDistribution;
use std::collections::{BTreeMap, HashMap, HashSet};

pub struct ConsistentHashing {
    nodes: HashMap<String, f64>,
    ring: Vec<u64>,
    node_replicas: u16,
    hash_to_node: BTreeMap<u64, String>,
    node_to_hashes: BTreeMap<String, HashSet<u64>>,
}

impl ConsistentHashing {
    pub fn new(nodes: HashMap<String, f64>, replica: u16) -> Self {
        let (ring, node_hash_to_id, node_id_to_hashes) =
            build_ring(nodes.keys().map(String::clone).collect(), replica);
        Self {
            nodes,
            ring,
            node_replicas: replica,
            hash_to_node: node_hash_to_id,
            node_to_hashes: node_id_to_hashes,
        }
    }

    #[must_use]
    pub const fn nodes(&self) -> &HashMap<String, f64> {
        &self.nodes
    }
}

impl ShardDistribution for ConsistentHashing {
    fn add(&mut self, node: String, available_resources: f64) {
        let hashes = add_node(&node, self.node_replicas);
        for hash in hashes {
            self.ring.push(hash);
            self.hash_to_node.insert(hash, node.clone());
            self.node_to_hashes
                .entry(node.clone())
                .or_default()
                .insert(hash);
        }
        self.nodes.insert(node, available_resources);
        self.ring.sort_unstable();
    }

    fn update(&mut self, node: String, available_resources: f64) {
        if let Some(avail) = self.nodes.get_mut(&node) {
            *avail = available_resources;
        }
    }

    fn remove(&mut self, node: String) {
        if self.nodes.contains_key(&node) {
            for hash in self.node_to_hashes.get(&node).unwrap() {
                let ring_hash_to_idx: BTreeMap<u64, usize> =
                    self.ring.iter().enumerate().map(|(i, v)| (*v, i)).collect();
                self.ring.remove(*ring_hash_to_idx.get(hash).unwrap());
                self.ring.retain(|e| *e != *hash);
                self.hash_to_node.remove(hash);
            }
            self.node_to_hashes.remove(&node);
            self.nodes.remove(&node);
        }
    }

    fn distribute(
        &mut self,
        entity: String,
        entity_replica: Option<u16>,
        consumed_resources: Option<f64>,
    ) -> HashSet<String> {
        if self.ring.is_empty() || entity_replica.is_none() {
            return HashSet::new();
        }
        let mut nodes = HashSet::new();
        let mut ring = self.ring.clone();
        if let Some(consumed_resources) = consumed_resources {
            // remove servers with not enough resources
            self.nodes
                .iter()
                .filter(|(_, avail)| **avail < consumed_resources)
                .for_each(|(node, _)| {
                    for hash in self.node_to_hashes.get(node).unwrap() {
                        let ring_hash_to_idx: BTreeMap<u64, usize> =
                            ring.iter().enumerate().map(|(i, v)| (*v, i)).collect();
                        ring.remove(*ring_hash_to_idx.get(hash).unwrap());
                    }
                });
        }
        for r in 0..entity_replica.unwrap() {
            if ring.is_empty() {
                break;
            }
            let entity_hash = hash(&format!("{entity}-{r}"));
            let node_hash = search_node(&ring, entity_hash);
            let node = self.hash_to_node.get(&node_hash).unwrap().clone();
            nodes.insert(node.clone());
            // remove nodes with vnodes from the ring
            for hash in self.node_to_hashes.get(&node).unwrap() {
                let ring_hash_to_idx: BTreeMap<u64, usize> =
                    ring.iter().enumerate().map(|(i, v)| (*v, i)).collect();
                ring.remove(*ring_hash_to_idx.get(hash).unwrap());
            }
            if let Some(consumed) = consumed_resources {
                if let Some(avail) = self.nodes.get_mut(&node) {
                    *avail -= consumed;
                }
            }
        }
        nodes
    }
}

fn search_node(nodes: &[u64], target: u64) -> u64 {
    assert!(!nodes.is_empty(), "No nodes found");
    let mid_idx = nodes.len() / 2;
    let mid = nodes[mid_idx];
    if target == mid {
        return mid;
    }
    let left = &nodes[..mid_idx];
    let right = &nodes[mid_idx..];
    if left.is_empty() {
        return right[0];
    } else if right.is_empty() {
        return left[0];
    }
    if target < mid {
        search_node(left, target)
    } else {
        search_node(right, target)
    }
}

type Ring = (
    Vec<u64>,
    BTreeMap<u64, String>,
    BTreeMap<String, HashSet<u64>>,
);

/// Build the ring by adding several virtual nodes to get a more uniform distribution.
#[must_use]
pub fn build_ring(nodes: Vec<String>, replicas: u16) -> Ring {
    let mut ring = vec![];
    let mut node_hash_to_id = BTreeMap::new();
    let mut node_id_to_hashes: BTreeMap<String, HashSet<u64>> = BTreeMap::new();

    for node in nodes {
        let hashes = add_node(&node, replicas);
        for hash in hashes {
            ring.push(hash);
            node_hash_to_id.insert(hash, node.clone());
            node_id_to_hashes
                .entry(node.clone())
                .or_default()
                .insert(hash);
        }
    }
    ring.sort_unstable();
    (ring, node_hash_to_id, node_id_to_hashes)
}

/// Add a node with virtual nodes.
fn add_node(node: &str, replicas: u16) -> Vec<u64> {
    let mut hashes = vec![];
    for r in 0..replicas {
        let hash = hash(&format!("{node}-{r}"));
        hashes.push(hash);
    }
    hashes
}

// add test module
#[cfg(test)]
mod tests {
    use crate::consistent_hashing::ConsistentHashing;
    use crate::shard_distribution::ShardDistribution;
    use std::collections::HashMap;

    #[test]
    fn basic() {
        let nodes: HashMap<String, f64> = vec![
            ("node1".to_string(), 15_f64),
            ("node2".to_string(), 25_f64),
            ("node3".to_string(), 30_f64),
            ("node4".to_string(), 35_f64),
            ("node5".to_string(), 42_f64),
        ]
        .into_iter()
        .collect();
        let mut hasher = ConsistentHashing::new(nodes, 5);
        let nodes = hasher.distribute("key1".to_string(), Some(3), Some(10_f64));
        let mut nodes = nodes.iter().cloned().collect::<Vec<String>>();
        nodes.sort_unstable();
        println!("key1 {nodes:?}");
        println!("nodes {:?}", hasher.nodes());
        println!();
        assert_eq!(nodes.len(), 3);

        let node = nodes.last().unwrap().clone();
        hasher.remove(node.clone());
        let nodes = hasher.distribute("key1".to_string(), Some(3), None);
        println!("{node} removed key1 {nodes:?}");
        println!("nodes {:?}", hasher.nodes());
        println!();
        assert!(!nodes.contains(&node));

        let nodes = hasher.distribute("key2".to_string(), None, None);
        println!("key2 no replicas {nodes:?}");
        println!("nodes {:?}", hasher.nodes());
        println!();
        assert!(nodes.is_empty());

        hasher.add("node6".to_string(), 10_f64);
        let nodes = hasher.distribute("key2".to_string(), Some(3), Some(10_f64));
        println!("node6 added key2 {nodes:?}");
        println!();
        println!("nodes {:?}", hasher.nodes());
        assert_eq!(nodes.len(), 3);

        hasher.remove("node6".to_string());
        hasher.remove("node5".to_string());
        hasher.remove("node4".to_string());
        hasher.remove("node3".to_string());
        let nodes = hasher.distribute("key3".to_string(), Some(3), Some(1_f64));
        println!("node[3-6] removed key3 {nodes:?}");
        println!("nodes {:?}", hasher.nodes());
        println!();
        assert_eq!(nodes.len(), 2);

        let nodes = hasher.distribute("key4".to_string(), Some(3), Some(50_f64));
        println!("use too many resources key4 {nodes:?}");
        println!("nodes {:?}", hasher.nodes());
        println!();
        assert_eq!(nodes.len(), 0);

        hasher.remove("node2".to_string());
        let nodes = hasher.distribute("key3".to_string(), Some(3), None);
        println!("node2 removed, lookup key3 {nodes:?}");
        println!("nodes {:?}", hasher.nodes());
        println!();
        assert_eq!(nodes.len(), 1);

        hasher.remove("node1".to_string());
        let nodes = hasher.distribute("key3".to_string(), Some(3), Some(5_f64));
        println!("node1 removed key3 {nodes:?}");
        println!("nodes {:?}", hasher.nodes());
        assert_eq!(nodes.len(), 0);
    }
}
