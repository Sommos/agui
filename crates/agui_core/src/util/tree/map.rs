use std::{
    collections::VecDeque,
    ops::{Index, IndexMut},
};

use slotmap::{
    hop::{Iter, IterMut},
    HopSlotMap,
};

pub struct TreeMap<K, V>
where
    K: slotmap::Key,
{
    nodes: HopSlotMap<K, TreeNode<K, V>>,
}

impl<K, V> Default for TreeMap<K, V>
where
    K: slotmap::Key,
{
    fn default() -> Self {
        Self {
            nodes: HopSlotMap::default(),
        }
    }
}

#[derive(Debug)]
pub struct TreeNode<K, V>
where
    K: slotmap::Key,
{
    depth: usize,
    value: Option<V>,

    parent: Option<K>,
    children: Vec<K>,
}

impl<K, V> TreeNode<K, V>
where
    K: slotmap::Key,
{
    pub fn depth(&self) -> usize {
        self.depth
    }

    pub fn value(&self) -> &V {
        self.value.as_ref().expect("node is currently in use")
    }

    pub fn value_mut(&mut self) -> &mut V {
        self.value.as_mut().expect("node is currently in use")
    }

    pub fn parent(&self) -> Option<K> {
        self.parent
    }

    pub fn children(&self) -> &[K] {
        &self.children
    }
}

impl<K, V> TreeMap<K, V>
where
    K: slotmap::Key,
{
    pub fn contains(&self, node_id: K) -> bool {
        self.nodes.contains_key(node_id)
    }

    pub fn get_depth(&self, node_id: K) -> Option<usize> {
        self.nodes.get(node_id).map(|node| node.depth)
    }

    pub(super) fn clear(&mut self) {
        self.nodes.clear();
    }

    pub(super) fn add(&mut self, parent_id: Option<K>, value: V) -> K {
        let node_id = self.nodes.insert(TreeNode {
            depth: 0,
            value: Some(value),
            parent: parent_id,
            children: Vec::new(),
        });

        self.propagate_node(parent_id, node_id);

        node_id
    }

    pub(super) fn remove(&mut self, node_id: K) -> Option<V> {
        if let Some(mut node) = self.nodes.remove(node_id) {
            if let Some(parent_id) = node.parent {
                if let Some(parent) = self.nodes.get_mut(parent_id) {
                    // Remove the child from its parent
                    parent.children.remove(
                        parent
                            .children
                            .iter()
                            .position(|child_id| node_id == *child_id)
                            .expect("unable to find child in removed node's parent"),
                    );
                }
            }

            Some(node.value.take().expect("node is currently in use"))
        } else {
            None
        }
    }

    /// Moves a node from one parent to another.
    ///
    /// Returns `true` if the node was moved, `false` if the node was already a child of the new parent.
    pub(super) fn reparent(&mut self, new_parent_id: Option<K>, node_id: K) -> bool {
        if let Some(node) = self.nodes.get(node_id) {
            if let Some(parent_id) = node.parent {
                if let Some(parent) = self.nodes.get_mut(parent_id) {
                    let child_idx = parent
                        .children
                        .iter()
                        .position(|child_id| node_id == *child_id)
                        .expect("unable to find child in removed node's parent");

                    // If the node isn't being moved to an entirely new parent
                    if Some(parent_id) == new_parent_id {
                        // If the widget is already the last child in the parent, don't do anything
                        if child_idx == parent.children.len() - 1 {
                            return false;
                        }

                        parent.children.remove(child_idx);

                        parent.children.push(node_id);

                        return false;
                    } else {
                        // Remove the child from its parent
                        parent.children.remove(child_idx);
                    }
                }
            }

            self.propagate_node(new_parent_id, node_id);
        }

        true
    }

    fn propagate_node(&mut self, parent_id: Option<K>, node_id: K) {
        let mut new_depth = 0;

        if let Some(parent_id) = parent_id {
            if let Some(parent) = self.nodes.get_mut(parent_id) {
                new_depth = parent.depth + 1;

                parent.children.push(node_id);
            } else {
                panic!("cannot add a node to a parent that doesn't exist");
            }
        }

        let node = &mut self.nodes[node_id];

        node.parent = parent_id;

        if node.depth != new_depth {
            let diff: i32 = (new_depth as i32) - (node.depth as i32);

            node.depth = new_depth;

            // If the node had children, propagate the depth difference
            if !node.children.is_empty() {
                let mut queue = VecDeque::from(node.children.clone());

                while let Some(child_id) = queue.pop_front() {
                    let child = self
                        .nodes
                        .get_mut(child_id)
                        .expect("unable to update child's depth, as it's not in the tree");

                    child.depth = ((child.depth as i32) + diff) as usize;

                    queue.extend(child.children.iter());
                }
            }
        }
    }

    pub fn with<F, R>(&mut self, node_id: K, func: F) -> Option<R>
    where
        F: FnOnce(&mut TreeMap<K, V>, &mut V) -> R,
    {
        if let Some(mut value) = self.take(node_id) {
            let ret = func(self, &mut value);

            self.replace(node_id, value);

            Some(ret)
        } else {
            None
        }
    }

    pub fn take(&mut self, node_id: K) -> Option<V> {
        self.nodes
            .get_mut(node_id)
            .map(|node| node.value.take().expect("node is currently in use"))
    }

    pub fn replace(&mut self, node_id: K, value: V) {
        self.nodes
            .get_mut(node_id)
            .map(|node| node.value.replace(value));
    }

    pub fn get_node(&self, node_id: K) -> Option<&TreeNode<K, V>> {
        self.nodes.get(node_id)
    }

    pub fn get_node_mut(&mut self, node_id: K) -> Option<&mut TreeNode<K, V>> {
        self.nodes.get_mut(node_id)
    }

    pub fn get(&self, node_id: K) -> Option<&V> {
        self.get_node(node_id).map(|node| node.value())
    }

    pub fn get_mut(&mut self, node_id: K) -> Option<&mut V> {
        self.get_node_mut(node_id).map(|node| node.value_mut())
    }

    pub fn get_parent(&self, node_id: K) -> Option<K> {
        self.get_node(node_id).and_then(|node| node.parent())
    }

    pub fn get_child(&self, node_id: K, idx: usize) -> Option<K> {
        self.get_node(node_id)
            .and_then(|node| node.children.get(idx).copied())
    }

    pub fn get_children(&self, node_id: K) -> Option<&Vec<K>> {
        self.get_node(node_id).map(|node| &node.children)
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn iter(&self) -> Iter<K, TreeNode<K, V>> {
        self.nodes.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<K, TreeNode<K, V>> {
        self.nodes.iter_mut()
    }

    pub fn iter_down_from(&self, node_id: K) -> impl Iterator<Item = K> + '_ {
        DownwardIterator {
            tree: self,
            node_id: Some(node_id),
            first: true,
        }
    }

    #[allow(dead_code)]
    pub fn iter_up_from(&self, node_id: K) -> impl Iterator<Item = K> + '_ {
        UpwardIterator {
            tree: self,
            node_id: Some(node_id),
            first: true,
        }
    }

    pub fn iter_subtree<'a, F>(&'a self, node_id: K, filter: F) -> impl Iterator<Item = K> + 'a
    where
        F: Fn(K) -> bool + 'a,
    {
        SubtreeIterator {
            tree: self,
            root_node_id: node_id,
            node_id: Some(node_id),
            first: true,
            filter,
        }
    }

    pub fn iter_parents(&self, node_id: K) -> impl Iterator<Item = K> + '_ {
        ParentIterator {
            tree: self,
            node_id: Some(node_id),
        }
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn has_child(&self, node_id: K, child_id: K) -> bool {
        // Make sure they're actually in the tree
        if !self.nodes.contains_key(node_id) || !self.nodes.contains_key(child_id) {
            return false;
        }

        let child_depth = self.nodes[child_id].depth;

        // If the node's depth is below the child, it's impossible for the child to be in the parent
        if self.nodes[node_id].depth >= child_depth {
            return false;
        }

        for node_id in self.iter_down_from(node_id) {
            let node = self.nodes.get(node_id).expect("tree broken");

            // If we reach a depth lower than the child, bail, because the child won't be found. We do
            // not do an equality check, here, because we may find the child as a sibling
            if node.depth > child_depth {
                return false;
            }

            // The child exists under the parent
            if node_id == child_id {
                return true;
            }
        }

        false
    }

    pub fn is_first_child(&self, node_id: K) -> bool {
        if let Some(parent_id) = self.get_parent(node_id) {
            if let Some(parent) = self.nodes.get(parent_id) {
                return parent
                    .children
                    .first()
                    .map_or(true, |child_id| *child_id == node_id);
            }
        }

        true
    }

    pub fn is_last_child(&self, node_id: K) -> bool {
        if let Some(parent_id) = self.get_parent(node_id) {
            if let Some(parent) = self.nodes.get(parent_id) {
                return parent
                    .children
                    .last()
                    .map_or(true, |child_id| *child_id == node_id);
            }
        }

        true
    }

    pub fn get_deepest_child(&self, mut current_node_id: Option<K>) -> Option<K> {
        while let Some(node_id) = current_node_id {
            if let Some(node) = self.nodes.get(node_id) {
                if let Some(last_child) = node.children.last() {
                    current_node_id = Some(*last_child);
                } else {
                    // If it has no children, this is the last node in the tree
                    break;
                }
            }
        }

        current_node_id
    }

    #[allow(clippy::unused_self)]
    pub fn get_next_sibling(&self, parent_id: K, sibling_id: K) -> Option<K> {
        if let Some(parent) = self.nodes.get(parent_id) {
            let mut children = parent.children.iter();

            while let Some(child_id) = children.next() {
                if *child_id == sibling_id {
                    let child_id = children.next();

                    if let Some(child_id) = child_id {
                        return Some(*child_id);
                    }

                    return None;
                }
            }
        }

        None
    }

    #[allow(clippy::unused_self)]
    pub fn get_prev_sibling(&self, parent_id: K, sibling_id: K) -> Option<K> {
        if let Some(parent) = self.nodes.get(parent_id) {
            let mut last_child_id = None;

            for child_id in &parent.children {
                if *child_id == sibling_id {
                    return last_child_id;
                }

                last_child_id = Some(*child_id);
            }
        }

        None
    }

    /// Returns any nodes that do not have any other node as a parent.
    pub fn filter_topmost<I>(&self, nodes: I) -> Vec<K>
    where
        I: Iterator<Item = K>,
    {
        let mut topmost = Vec::new();

        'main: for key in nodes {
            let tree_node = match self.nodes.get(key) {
                Some(widget) => widget,
                None => continue,
            };

            let node_depth = tree_node.depth;

            let mut i = 0;

            while i < topmost.len() {
                let (dirty_id, dirty_depth) = topmost[i];

                // If they're at the same depth, bail. No reason to check if they're children.
                if node_depth != dirty_depth {
                    if node_depth > dirty_depth {
                        // If the node is a child of one of the `topmost` nodes, bail
                        if self.has_child(dirty_id, key) {
                            continue 'main;
                        }
                    } else {
                        // If the node is a parent of a node already in the `topmost` vec, remove it
                        if self.has_child(key, dirty_id) {
                            topmost.remove(i);
                            continue;
                        }
                    }
                }

                i += 1;
            }

            topmost.push((key, node_depth));
        }

        topmost
            .into_iter()
            .map(|(widget_id, _)| widget_id)
            .collect::<Vec<_>>()
    }
}

impl<K, V> Index<K> for TreeMap<K, V>
where
    K: slotmap::Key,
{
    type Output = V;

    fn index(&self, key: K) -> &Self::Output {
        self.nodes[key]
            .value
            .as_ref()
            .expect("node is currently in use")
    }
}

impl<K, V> IndexMut<K> for TreeMap<K, V>
where
    K: slotmap::Key,
{
    fn index_mut(&mut self, key: K) -> &mut Self::Output {
        self.nodes[key]
            .value
            .as_mut()
            .expect("node is currently in use")
    }
}

pub struct DownwardIterator<'a, K, V>
where
    K: slotmap::Key,
{
    pub(super) tree: &'a TreeMap<K, V>,
    pub(super) node_id: Option<K>,
    pub(super) first: bool,
}

impl<'a, K, V> Iterator for DownwardIterator<'a, K, V>
where
    K: slotmap::Key,
{
    type Item = K;

    fn next(&mut self) -> Option<K> {
        if self.first {
            self.first = false;
            return self.node_id;
        }

        if let Some(node_id) = self.node_id {
            // Grab the node from the tree
            if let Some(node) = self.tree.nodes.get(node_id) {
                // Grab the first child node
                if let Some(child_id) = node.children.first() {
                    self.node_id = Some(*child_id);

                    return self.node_id;
                }

                let mut current_parent = node.parent;
                let mut after_child_id = node_id;

                loop {
                    // If we have no children, return the sibling after the node_id
                    if let Some(parent_node_id) = current_parent {
                        if let Some(sibling_id) =
                            self.tree.get_next_sibling(parent_node_id, after_child_id)
                        {
                            self.node_id = Some(sibling_id);
                            break;
                        } else {
                            // Move up to to the parent to check its next child
                            current_parent = self.tree.nodes[parent_node_id].parent;

                            // Set after_child_id to parent_node_id so it's skipped
                            after_child_id = parent_node_id;
                        }
                    } else {
                        // Has no parent. Bail.
                        self.node_id = None;
                        break;
                    }
                }
            } else {
                // If the node doesn't exist in the tree, then there's nothing to iterate
                self.node_id = None;
            }
        }

        self.node_id
    }
}

pub struct SubtreeIterator<'a, K, V, F>
where
    K: slotmap::Key,
{
    pub(super) tree: &'a TreeMap<K, V>,
    pub(super) root_node_id: K,
    pub(super) node_id: Option<K>,
    pub(super) first: bool,
    pub(super) filter: F,
}

impl<'a, K, V, F> Iterator for SubtreeIterator<'a, K, V, F>
where
    K: slotmap::Key,
    F: Fn(K) -> bool,
{
    type Item = K;

    fn next(&mut self) -> Option<K> {
        if self.first {
            self.first = false;

            if let Some(node_id) = self.node_id {
                if (self.filter)(node_id) {
                    return self.node_id;
                } else {
                    self.node_id = None;
                }
            }
        }

        if let Some(node_id) = self.node_id {
            // Grab the node from the tree
            if let Some(node) = self.tree.nodes.get(node_id) {
                // Grab the first child node
                if let Some(child_id) = node.children.first() {
                    // If the child passes the filter, return it
                    if (self.filter)(*child_id) {
                        self.node_id = Some(*child_id);

                        return self.node_id;
                    }
                }

                let mut current_parent = node.parent;
                let mut after_child_id = node_id;

                'find_child: loop {
                    // We're iterating a subtree, so we don't want to go above the defined root node
                    if after_child_id == self.root_node_id {
                        self.node_id = None;
                        break 'find_child;
                    }

                    // If we have no children, return the sibling after the node_id
                    if let Some(parent_node_id) = current_parent {
                        // Check each sibling of the parent for ones that pass the filter
                        while let Some(sibling_id) =
                            self.tree.get_next_sibling(parent_node_id, after_child_id)
                        {
                            if (self.filter)(sibling_id) {
                                self.node_id = Some(sibling_id);
                                break 'find_child;
                            }

                            after_child_id = sibling_id;
                        }

                        // Move up to to the parent to check its next child
                        current_parent = self.tree.nodes[parent_node_id].parent;

                        // Set after_child_id to parent_node_id so it's skipped
                        after_child_id = parent_node_id;
                    } else {
                        // Has no parent. Bail.
                        self.node_id = None;
                        break 'find_child;
                    }
                }
            } else {
                // If the node doesn't exist in the tree, then there's nothing to iterate
                self.node_id = None;
            }
        }

        self.node_id
    }
}

pub struct UpwardIterator<'a, K, V>
where
    K: slotmap::Key,
{
    pub(super) tree: &'a TreeMap<K, V>,
    pub(super) node_id: Option<K>,
    pub(super) first: bool,
}

impl<'a, K, V> Iterator for UpwardIterator<'a, K, V>
where
    K: slotmap::Key,
{
    type Item = K;

    fn next(&mut self) -> Option<K> {
        if self.first {
            self.first = false;
            return self.node_id;
        }

        if let Some(node_id) = self.node_id {
            // Grab the node from the tree
            if let Some(node) = self.tree.nodes.get(node_id) {
                if let Some(parent_node_id) = node.parent {
                    if let Some(sibling_id) = self.tree.get_prev_sibling(parent_node_id, node_id) {
                        self.node_id = self.tree.get_deepest_child(Some(sibling_id));
                    } else {
                        self.node_id = node.parent;
                    }
                } else {
                    // TreeNode doesn't have a parent, so we're at the root.
                    self.node_id = None;
                }
            } else {
                // If the node doesn't exist in the tree, then there's nothing to iterate
                self.node_id = None;
            }
        }

        self.node_id
    }
}

pub struct ParentIterator<'a, K, V>
where
    K: slotmap::Key,
{
    tree: &'a TreeMap<K, V>,
    node_id: Option<K>,
}

impl<'a, K, V> Iterator for ParentIterator<'a, K, V>
where
    K: slotmap::Key,
{
    type Item = K;

    fn next(&mut self) -> Option<K> {
        if let Some(node_id) = self.node_id {
            // Grab the node from the tree
            if let Some(node) = self.tree.nodes.get(node_id) {
                self.node_id = node.parent;
            } else {
                // If the node doesn't exist in the tree, then there's nothing to iterate
                self.node_id = None;
            }
        }

        self.node_id
    }
}

pub struct ChildIterator<'a, K, V>
where
    K: slotmap::Key,
{
    pub(super) tree: &'a TreeMap<K, V>,
    pub(super) node_id: K,
    pub(super) current_child_id: Option<K>,
    pub(super) first: bool,
}

impl<'a, K, V> Iterator for ChildIterator<'a, K, V>
where
    K: slotmap::Key,
{
    type Item = K;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(node) = self.tree.nodes.get(self.node_id) {
            let mut children = node.children.iter();

            if let Some(current_child_id) = self.current_child_id {
                self.current_child_id = None;

                while let Some(child_id) = children.next() {
                    if *child_id == current_child_id {
                        let child_id = children.next();

                        if let Some(child_id) = child_id {
                            self.current_child_id = Some(*child_id);
                        } else {
                            self.current_child_id = None;
                        }

                        break;
                    }
                }
            } else if self.first {
                self.first = false;

                let child_id = children.next();

                if let Some(child_id) = child_id {
                    self.current_child_id = Some(*child_id);
                } else {
                    self.current_child_id = None;
                }
            }

            return self.current_child_id;
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use crate::element::ElementId;

    use super::TreeMap;

    #[test]
    fn hierarchy() {
        let mut tree: TreeMap<ElementId, usize> = TreeMap::default();

        let root_id = tree.add(None, 0);

        let child_1 = tree.add(Some(root_id), 1);
        let child_1_1 = tree.add(Some(child_1), 2);
        let child_1_1_1 = tree.add(Some(child_1_1), 3);
        let child_1_2 = tree.add(Some(child_1), 4);
        let child_1_3 = tree.add(Some(child_1), 5);

        let child_2 = tree.add(Some(root_id), 6);

        let child_3 = tree.add(Some(root_id), 7);
        let child_3_1 = tree.add(Some(child_3), 8);

        assert!(
            tree.is_first_child(child_1),
            "child_1 is the first child of the parent"
        );
        assert!(
            !tree.is_last_child(child_1),
            "child_1 is not the last child of the parent"
        );

        assert!(
            tree.is_first_child(child_1_1),
            "child_1_1 is the first child of the parent"
        );
        assert!(
            !tree.is_last_child(child_1_1),
            "child_1_1 is not the last child of the parent"
        );

        assert!(
            tree.is_first_child(child_1_1_1),
            "child_1_1_1 is the first child of the parent"
        );
        assert!(
            tree.is_last_child(child_1_1_1),
            "child_1_1_1 is the last child of the parent"
        );

        assert!(
            !tree.is_first_child(child_1_2),
            "child_1_2 is not the first child of the parent"
        );
        assert!(
            !tree.is_last_child(child_1_2),
            "child_1_2 is not the last child of the parent"
        );

        assert!(
            !tree.is_first_child(child_1_3),
            "child_1_3 is not the first child of the parent"
        );
        assert!(
            tree.is_last_child(child_1_3),
            "child_1_3 is the last child of the parent"
        );

        assert!(
            !tree.is_first_child(child_2),
            "child_2 is not the first child of the parent"
        );
        assert!(
            !tree.is_last_child(child_2),
            "child_2 is not the last child of the parent"
        );

        assert!(
            !tree.is_first_child(child_3),
            "child_3 is not the first child of the parent"
        );
        assert!(
            tree.is_last_child(child_3),
            "child_3 is the last child of the parent"
        );

        assert!(
            tree.is_first_child(child_3_1),
            "child_3_1 is the first child of the parent"
        );
        assert!(
            tree.is_last_child(child_3_1),
            "child_3_1 is the last child of the parent"
        );
    }

    #[test]
    fn downward_iter() {
        let mut tree: TreeMap<ElementId, usize> = TreeMap::default();

        let root_id = tree.add(None, 0);

        let child_1 = tree.add(Some(root_id), 1);
        let child_1_1 = tree.add(Some(child_1), 2);
        let child_1_1_1 = tree.add(Some(child_1_1), 3);
        let child_1_2 = tree.add(Some(child_1), 4);
        let child_1_3 = tree.add(Some(child_1), 5);

        let child_2 = tree.add(Some(root_id), 6);

        let child_3 = tree.add(Some(root_id), 7);
        let child_3_1 = tree.add(Some(child_3), 8);

        let mut iter = tree.iter_down_from(root_id);

        assert_eq!(
            iter.next(),
            Some(root_id),
            "downward iterator's first element must be the root node"
        );
        assert_eq!(
            iter.next(),
            Some(child_1),
            "downward iterator should have returned child_1"
        );
        assert_eq!(
            iter.next(),
            Some(child_1_1),
            "downward iterator should have returned child_1_1"
        );
        assert_eq!(
            iter.next(),
            Some(child_1_1_1),
            "downward iterator should have returned child_1_1_1"
        );
        assert_eq!(
            iter.next(),
            Some(child_1_2),
            "downward iterator should have returned child_1_2"
        );
        assert_eq!(
            iter.next(),
            Some(child_1_3),
            "downward iterator should have returned child_1_3"
        );
        assert_eq!(
            iter.next(),
            Some(child_2),
            "downward iterator should have returned child_2"
        );
        assert_eq!(
            iter.next(),
            Some(child_3),
            "downward iterator should have returned child_3"
        );
        assert_eq!(
            iter.next(),
            Some(child_3_1),
            "downward iterator should have returned child_3_1"
        );
        assert_eq!(
            iter.next(),
            None,
            "downward iterator should have returned None"
        );
        assert_eq!(
            iter.next(),
            None,
            "downward iterator should have returned None"
        );

        let mut iter = tree.iter_down_from(child_2);

        assert_eq!(
            iter.next(),
            Some(child_2),
            "downward iterator should have returned child_2"
        );

        assert_eq!(
            iter.next(),
            Some(child_3),
            "downward iterator should have returned child_3"
        );
        assert_eq!(
            iter.next(),
            Some(child_3_1),
            "downward iterator should have returned child_3_1"
        );
        assert_eq!(
            iter.next(),
            None,
            "downward iterator should have returned None"
        );
        assert_eq!(
            iter.next(),
            None,
            "downward iterator should have returned None"
        );

        let mut iter = tree.iter_down_from(child_3);

        assert_eq!(
            iter.next(),
            Some(child_3),
            "downward iterator should have returned child_3"
        );
        assert_eq!(
            iter.next(),
            Some(child_3_1),
            "downward iterator should have returned child_3_1"
        );
        assert_eq!(
            iter.next(),
            None,
            "downward iterator should have returned None"
        );
    }

    #[test]
    fn upward_iter() {
        let mut tree: TreeMap<ElementId, usize> = TreeMap::default();

        let root_id = tree.add(None, 0);

        let child_1 = tree.add(Some(root_id), 1);
        let child_1_1 = tree.add(Some(child_1), 2);
        let child_1_1_1 = tree.add(Some(child_1_1), 3);
        let child_1_2 = tree.add(Some(child_1), 4);
        let child_1_3 = tree.add(Some(child_1), 5);

        let child_2 = tree.add(Some(root_id), 6);

        let child_3 = tree.add(Some(root_id), 7);
        let child_3_1 = tree.add(Some(child_3), 8);

        let mut iter = tree.iter_up_from(child_3_1);

        assert_eq!(
            iter.next(),
            Some(child_3_1),
            "upward iterator should have returned child_3_1"
        );
        assert_eq!(
            iter.next(),
            Some(child_3),
            "upward iterator should have returned child_3"
        );
        assert_eq!(
            iter.next(),
            Some(child_2),
            "upward iterator should have returned child_2"
        );
        assert_eq!(
            iter.next(),
            Some(child_1_3),
            "upward iterator should have returned child_1_3"
        );
        assert_eq!(
            iter.next(),
            Some(child_1_2),
            "upward iterator should have returned child_1_2"
        );
        assert_eq!(
            iter.next(),
            Some(child_1_1_1),
            "upward iterator should have returned child_1_1_1"
        );
        assert_eq!(
            iter.next(),
            Some(child_1_1),
            "upward iterator should have returned child_1_1"
        );
        assert_eq!(
            iter.next(),
            Some(child_1),
            "upward iterator should have returned child_1"
        );
        assert_eq!(
            iter.next(),
            Some(root_id),
            "upward iterator should have returned the root node"
        );
        assert_eq!(
            iter.next(),
            None,
            "upward iterator should have returned None"
        );
        assert_eq!(
            iter.next(),
            None,
            "upward iterator should have returned None"
        );

        let mut iter = tree.iter_up_from(child_1_2);

        assert_eq!(
            iter.next(),
            Some(child_1_2),
            "upward iterator should have returned child_1_2"
        );
        assert_eq!(
            iter.next(),
            Some(child_1_1_1),
            "upward iterator should have returned child_1_1_1"
        );
        assert_eq!(
            iter.next(),
            Some(child_1_1),
            "upward iterator should have returned child_1_1"
        );
        assert_eq!(
            iter.next(),
            Some(child_1),
            "upward iterator should have returned child_1"
        );
        assert_eq!(
            iter.next(),
            Some(root_id),
            "upward iterator should have returned the root node"
        );
        assert_eq!(
            iter.next(),
            None,
            "upward iterator should have returned None"
        );
    }

    #[test]
    fn subtree_iter() {
        let mut tree: TreeMap<ElementId, usize> = TreeMap::default();

        let root_id = tree.add(None, 0);

        let child_1 = tree.add(Some(root_id), 1);
        let child_1_1 = tree.add(Some(child_1), 2);
        let child_1_1_1 = tree.add(Some(child_1_1), 3);
        let child_1_2 = tree.add(Some(child_1), 4);
        let child_1_3 = tree.add(Some(child_1), 5);

        let child_2 = tree.add(Some(root_id), 6);

        let child_3 = tree.add(Some(root_id), 7);
        let child_3_1 = tree.add(Some(child_3), 8);

        let mut iter = tree.iter_subtree(child_1, |_| true);

        assert_eq!(
            iter.next(),
            Some(child_1),
            "subtree iterator's first element must be child_1"
        );
        assert_eq!(
            iter.next(),
            Some(child_1_1),
            "subtree iterator should have returned child_1_1"
        );
        assert_eq!(
            iter.next(),
            Some(child_1_1_1),
            "subtree iterator should have returned child_1_1_1"
        );
        assert_eq!(
            iter.next(),
            Some(child_1_2),
            "subtree iterator should have returned child_1_2"
        );
        assert_eq!(
            iter.next(),
            Some(child_1_3),
            "subtree iterator should have returned child_1_3"
        );
        assert_eq!(
            iter.next(),
            None,
            "subtree iterator should have returned None"
        );

        let mut iter = tree.iter_subtree(child_2, |_| true);

        assert_eq!(
            iter.next(),
            Some(child_2),
            "subtree iterator should have returned child_2"
        );
        assert_eq!(
            iter.next(),
            None,
            "subtree iterator should have returned None"
        );

        let mut iter = tree.iter_subtree(child_3, |_| true);

        assert_eq!(
            iter.next(),
            Some(child_3),
            "subtree iterator should have returned child_3"
        );
        assert_eq!(
            iter.next(),
            Some(child_3_1),
            "subtree iterator should have returned child_3_1"
        );
        assert_eq!(
            iter.next(),
            None,
            "subtree iterator should have returned None"
        );
    }

    #[test]
    fn depth_propagation() {
        let mut tree: TreeMap<ElementId, usize> = TreeMap::default();

        let root_id = tree.add(None, 0);

        let child_1 = tree.add(Some(root_id), 1);
        let child_1_1 = tree.add(Some(child_1), 2);
        let child_1_1_1 = tree.add(Some(child_1_1), 3);
        let child_1_2 = tree.add(Some(child_1), 4);
        let child_1_3 = tree.add(Some(child_1), 5);

        let child_2 = tree.add(Some(root_id), 6);

        let child_3 = tree.add(Some(root_id), 7);
        let child_3_1 = tree.add(Some(child_3), 8);

        assert_eq!(
            tree.get_depth(root_id),
            Some(0),
            "root node should have depth 0"
        );

        assert_eq!(
            tree.get_depth(child_1),
            Some(1),
            "child_1 should have depth 1"
        );
        assert_eq!(
            tree.get_depth(child_1_1),
            Some(2),
            "child_1_1 should have depth 2"
        );
        assert_eq!(
            tree.get_depth(child_1_1_1),
            Some(3),
            "child_1_1_1 should have depth 3"
        );
        assert_eq!(
            tree.get_depth(child_1_2),
            Some(2),
            "child_1_2 should have depth 2"
        );
        assert_eq!(
            tree.get_depth(child_1_3),
            Some(2),
            "child_1_3 should have depth 2"
        );

        assert_eq!(
            tree.get_depth(child_2),
            Some(1),
            "child_2 should have depth 1"
        );

        assert_eq!(
            tree.get_depth(child_3),
            Some(1),
            "child_3 should have depth 1"
        );
        assert_eq!(
            tree.get_depth(child_3_1),
            Some(2),
            "child_3_1 should have depth 2"
        );

        tree.reparent(Some(root_id), child_1_1);

        assert_eq!(
            tree.get_depth(root_id),
            Some(0),
            "root node should have depth 0"
        );

        assert_eq!(
            tree.get_depth(child_1),
            Some(1),
            "child_1 should have depth 1"
        );
        assert_eq!(
            tree.get_depth(child_1_1),
            Some(1),
            "child_1_1 should have depth 1"
        );
        assert_eq!(
            tree.get_depth(child_1_1_1),
            Some(2),
            "child_1_1_1 should have depth 2"
        );
        assert_eq!(
            tree.get_depth(child_1_2),
            Some(2),
            "child_1_2 should have depth 1"
        );
        assert_eq!(
            tree.get_depth(child_1_3),
            Some(2),
            "child_1_3 should have depth 2"
        );

        assert_eq!(
            tree.get_depth(child_2),
            Some(1),
            "child_2 should have depth 1"
        );

        assert_eq!(
            tree.get_depth(child_3),
            Some(1),
            "child_3 should have depth 1"
        );
        assert_eq!(
            tree.get_depth(child_3_1),
            Some(2),
            "child_3_1 should have depth 2"
        );
    }
}
