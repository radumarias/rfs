use std::collections::HashSet;

/// Shard distribution in a cluster of nodes based on resources.
pub trait ShardDistribution {
    /// Add a new node.
    ///
    /// `available_resources` available resources for the node used when distributing entities.
    fn add(&mut self, node: String, available_resources: f64);

    /// Update the metric of a node.
    ///
    /// `available_resources` available resources for the node used when distributing entities.
    fn update(&mut self, node: String, available_resources: f64);

    /// Remove a node.
    fn remove(&mut self, node: String);

    /// Get the nodes on which the entity resides.
    ///
    /// `consumed_resources` consumed resources for the entity.
    /// Nodes with the lower available resources are **NOT** selected.
    /// If is `None` then the entity is not distributed, this is useful to do lookup only.
    ///
    /// Returns a list of nodes based on entity replica count.
    /// If there are not enough nodes, it returns the max-available nodes.
    fn distribute(
        &mut self,
        entity: String,
        entity_replicas: Option<u16>,
        consumed_resources: Option<f64>,
    ) -> HashSet<String>;
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::consistent_hashing::ConsistentHashing;
    use crate::shard_distribution::ShardDistribution;

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
