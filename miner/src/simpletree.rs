pub trait Parenting {
    fn is_parent(&self, parent_id: &[u8]) -> bool;
}

#[derive(Debug, Default)]
pub struct TreeNode<T: Default + Parenting> {
    value: T,
    children: Vec<TreeNode<T>>,
}

impl<T: Default + Parenting> TreeNode<T> {
    /// Create a new tree node with the given value and no children
    pub fn new(value: T) -> Self {
        TreeNode {
            value,
            children: Vec::new(),
        }
    }

    /// Insert a new child node with the given value
    pub fn insert(&mut self, value: T) {
        self.children.push(TreeNode::new(value));
    }

    /// Remove all children nodes with the given value (recursively)
    #[allow(dead_code)]
    pub fn remove(&mut self, value: &T)
    where
        T: PartialEq,
    {
        self.children.retain_mut(|child| {
            child.remove(value);
            &child.value != value
        });
    }

    /*
    /// Calculate the depth of the tree from this node
    pub fn depth(&self) -> usize {
        let mut max_depth = 0;
        for child in &self.children {
            let child_depth = child.depth();
            if child_depth > max_depth {
                max_depth = child_depth;
            }
        }
        max_depth + 1
    }

    */

    /// Perform a depth-first search looking for the parent
    pub fn look_for_parent(&mut self, parent_id: &[u8]) -> Option<&mut TreeNode<T>> {
        if self.value.is_parent(parent_id) {
            return Some(self);
        }

        for child in &mut self.children {
            if let Some(found) = child.look_for_parent(parent_id) {
                return Some(found);
            }
        }
        None
    }

    /// Returns a reference to the deepest leaf node in the tree
    pub fn deepest_leafs(&self) -> Vec<&TreeNode<T>> {
        fn helper<'a, T: Default + Parenting>(
            node: &'a TreeNode<T>,
            depth: usize,
            max_depth: &mut usize,
            deepest: &mut Vec<&'a TreeNode<T>>,
        ) {
            if node.children.is_empty() {
                if depth > *max_depth {
                    *max_depth = depth;
                    deepest.clear();
                    deepest.push(node);
                } else if depth == *max_depth {
                    deepest.push(node);
                }
            } else {
                for child in &node.children {
                    helper(child, depth + 1, max_depth, deepest);
                }
            }
        }

        let mut max_depth = 0;
        let mut result = Vec::new();
        helper(self, 1, &mut max_depth, &mut result);
        result
    }

    /// Get the value of the node
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Get a reference to the children of the node
    pub fn children(&self) -> &Vec<TreeNode<T>> {
        &self.children
    }

    /// Get a mutable reference to the children of the node
    #[allow(dead_code)]
    pub fn children_mut(&mut self) -> &mut Vec<TreeNode<T>> {
        &mut self.children
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(PartialEq, Default, Debug)]
    struct Data {
        val: u32,
        parent_id: [u8; 1],
    }

    impl Data {
        fn new(val: u32, parent_id: [u8; 1]) -> Self {
            Data { val, parent_id }
        }
    }

    impl Parenting for Data {
        fn is_parent(&self, parent_id: &[u8]) -> bool {
            self.val as u8 == parent_id[0]
        }
    }

    #[test]
    fn test_tree_operations() {
        let mut root = TreeNode::new(Data::new(42, [0]));
        root.insert(Data::new(2, [42]));
        root.insert(Data::new(3, [42]));
        root.children_mut()[0].insert(Data::new(4, [2]));
        root.children_mut()[0].insert(Data::new(5, [2]));
        root.children_mut()[1].insert(Data::new(6, [3]));


        // Test look_for_parent
        assert_eq!(
            root.look_for_parent(&[3]).unwrap().value(),
            &Data::new(3, [42])
        );

        // Test remove
        root.remove(&Data::new(2, [42]));
        assert_eq!(root.children().len(), 1);
        assert_eq!(root.children()[0].value(), &Data::new(3, [42]));
    }

    #[test]
    fn test_deepest_leafs() {
        let mut root = TreeNode::new(Data::new(1, [0]));
        root.insert(Data::new(2, [1])); // First child of root
        root.insert(Data::new(3, [1])); // Second child of root

        // Add one deep branch to first child
        root.children_mut()[0].insert(Data::new(4, [2]));
        root.children_mut()[0].children_mut()[0].insert(Data::new(5, [4]));

        // Add equally deep branch to second child
        root.children_mut()[1].insert(Data::new(6, [3]));
        root.children_mut()[1].children_mut()[0].insert(Data::new(7, [6]));

        let deepest = root.deepest_leafs();

        let expected_values = vec![
            Data::new(5, [4]),
            Data::new(7, [6]),
        ];

        assert_eq!(deepest.len(), 2);
        assert!(deepest.iter().any(|n| *n.value() == expected_values[0]));
        assert!(deepest.iter().any(|n| *n.value() == expected_values[1]));
    }
}
