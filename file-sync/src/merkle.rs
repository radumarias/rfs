use std::error::Error;

#[derive(Debug, Clone)]
pub struct Tree {
    #[allow(dead_code)]
    root: Node,
}

impl Tree {
    /// # Panics
    ///
    pub fn new(chunks: Vec<&[u8]>) -> Result<Self, Box<dyn Error>> {
        let mut chunks = chunks;

        if chunks.is_empty() {
            return Err("There are 0 chunks provided".into());
        }

        if chunks.len() % 2 != 0 {
            let last = chunks.last().unwrap();
            chunks.push(*last);
        }

        let leaves = chunks
            .into_iter()
            .map(|d| Node {
                hash: blake3::hash(d).as_bytes().to_vec(),
                right: None,
                left: None,
            })
            .collect::<Vec<Node>>();

        let root = Self::build_tree(leaves);

        Ok(Self { root })
    }

    fn build_tree(mut leaves: Vec<Node>) -> Node {
        if leaves.len() == 1 {
            return leaves.remove(0);
        }

        let mut parent_nodes: Vec<Node> = Vec::new();

        while leaves.len() > 1 {
            let left_node = leaves.remove(0);
            let right_node = leaves.remove(0);

            let new_parent = Node::create_parent(left_node, right_node);

            parent_nodes.push(new_parent);
        }
        if !leaves.is_empty() {
            parent_nodes.extend(leaves);
        }

        Self::build_tree(parent_nodes)
    }
}

#[derive(Debug, Clone, Eq)]
pub struct Node {
    #[allow(dead_code)]
    right: Option<Box<Node>>,

    #[allow(dead_code)]
    left: Option<Box<Node>>,

    #[allow(dead_code)]
    hash: Vec<u8>,
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.hash.eq(&other.hash)
    }
}

impl Node {
    #[must_use]
    pub fn create_parent(left: Self, right: Self) -> Self {
        let mut hasher = blake3::Hasher::new();
        let combined_hash = hasher.update(&left.hash).update(&right.hash).finalize();

        Self {
            left: Some(Box::new(left)),
            right: Some(Box::new(right)),
            hash: combined_hash.as_bytes().to_vec(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::merkle::Tree;

    #[test]
    fn create_a_merkle_tree() {
        let data: Vec<&[u8]> = vec![b"This", b"creates", b"a", b"merkle", b"tree"];

        let m_tree_root = Tree::new(data).unwrap().root;

        let h5_clone = m_tree_root.clone().right.unwrap().left.unwrap();
        let h5 = m_tree_root.right.unwrap().right.unwrap();

        assert_eq!(h5, h5_clone);
    }
}
