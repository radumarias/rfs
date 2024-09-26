use std::{error::Error, fmt::Display};

#[derive(Debug, Clone)]
pub struct Tree {
    #[allow(dead_code)]
    root: Node,
}

impl Tree {
    /// # Panics
    ///
    pub fn new(chunks: Vec<&[u8]>) -> Result<Self, Box<dyn Error>> {
        if chunks.is_empty() {
            return Err("There are 0 chunks provided".into());
        }

        let leaves = chunks
            .into_iter()
            .enumerate()
            .map(|(i, d)| Node {
                hash: blake3::hash(d).as_bytes().to_vec(),
                right: None,
                left: None,
                chunk_id: Some(i),
            })
            .collect::<Vec<Node>>();

        let root = Self::build_tree(leaves);

        Ok(Self { root })
    }

    fn build_tree(mut leaves: Vec<Node>) -> Node {
        if leaves.len() == 1 {
            return leaves.remove(0);
        }

        let mid = (leaves.len() + 1) / 2;

        let (left_leaves, right_leaves) = leaves.split_at(mid);

        let left = Self::build_tree(left_leaves.to_vec());
        let right = Self::build_tree(right_leaves.to_vec());

        Node::create_parent(left, right)
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

    #[allow(dead_code)]
    chunk_id: Option<usize>,
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
            chunk_id: None,
        }
    }

    fn fmt_with_indent(&self, f: &mut std::fmt::Formatter<'_>, indent: usize) -> std::fmt::Result {
        for _ in 0..indent {
            write!(f, "    ")?;
        }

        // Display the first hash byte of the current node
        writeln!(f, "Node(hash: [{:?}..])", self.hash[0])?;

        if let Some(left) = &self.left {
            for _ in 0..indent {
                write!(f, "    ")?;
            }
            write!(f, "L: ")?;
            left.fmt_with_indent(f, indent + 1)?;
        }

        if let Some(right) = &self.right {
            for _ in 0..indent {
                write!(f, "    ")?;
            }
            write!(f, "R: ")?;
            right.fmt_with_indent(f, indent + 1)?;
        }

        Ok(())
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_with_indent(f, 0)
    }
}

#[cfg(test)]
mod tests {
    use crate::merkle::Tree;

    #[test]
    fn create_a_merkle_tree() {
        let data: Vec<&[u8]> = vec![b"This", b"creates", b"a", b"balanced", b"merkle", b"tree"];

        let m_tree_root = Tree::new(data);

        assert!(m_tree_root.is_ok());
    }
}
