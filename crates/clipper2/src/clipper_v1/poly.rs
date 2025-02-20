use super::types::*;


/// A port of the C# PolyNode class.
#[derive(Debug, Clone)]
pub struct PolyNode {
    /// Parent node.
    pub parent: Option<Box<PolyNode>>,
    /// The node's contour (polygon).
    pub polygon: Path,
    /// The index of this node among its parent's children.
    pub index: usize,
    /// The join type, as in the original.
    pub jointype: JoinType,
    /// The end type, as in the original.
    pub endtype: EndType,
    /// Child nodes.
    pub childs: Vec<PolyNode>,
    /// Indicates if this node represents an open path.
    pub is_open: bool,
}

impl PolyNode {
    /// Constructs a new, empty PolyNode.
    pub fn new() -> Self {
        Self {
            parent: None,
            polygon: Vec::new(),
            index: 0,
            jointype: JoinType::Square,
            endtype: EndType::ClosedPolygon,
            childs: Vec::new(),
            is_open: false,
        }
    }

    /// Returns the number of child nodes.
    pub fn child_count(&self) -> usize {
        self.childs.len()
    }

    /// Returns a reference to the node's contour (its polygon).
    pub fn contour(&self) -> &Path {
        &self.polygon
    }

    /// Adds a child to the node.
    /// Sets the child's parent and index.
    pub fn add_child(&mut self, mut child: PolyNode) {
        child.parent = Some(Box::new(self.clone()));
        child.index = self.childs.len();
        self.childs.push(child);
    }

    /// Returns the next node in a traversal.
    pub fn get_next(&self) -> Option<&PolyNode> {
        if !self.childs.is_empty() {
            Some(&self.childs[0])
        } else {
            self.get_next_sibling_up()
        }
    }

    /// Returns the next sibling up the parent chain.
    pub fn get_next_sibling_up(&self) -> Option<&PolyNode> {
        if let Some(ref parent) = self.parent {
            if self.index + 1 < parent.childs.len() {
                Some(&parent.childs[self.index + 1])
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Determines whether this node is a hole.
    pub fn is_hole(&self) -> bool {
        let mut result = false;
        let mut node = &self.parent;
        while let Some(ref p) = node {
            result = !result;
            node = &p.parent;
        }
        result
    }
}

/// A port of the C# PolyTree class.
#[derive(Debug, Clone)]
pub struct PolyTree {
    /// The root node. In the C# version PolyTree inherits from PolyNode.
    pub root: PolyNode,
    /// A list of all nodes in the tree.
    pub all_polys: Vec<PolyNode>,
}

impl PolyTree {
    /// Constructs a new, empty PolyTree.
    pub fn new() -> Self {
        Self {
            root: PolyNode::new(),
            all_polys: Vec::new(),
        }
    }

    /// Clears the PolyTree.
    pub fn clear(&mut self) {
        self.root.childs.clear();
        self.all_polys.clear();
    }

    /// Returns the total number of nodes in the tree.
    pub fn total(&self) -> usize {
        self.all_polys.len()
    }

    /// Returns the first node in a traversal.
    pub fn get_first(&self) -> Option<&PolyNode> {
        if !self.root.childs.is_empty() {
            Some(&self.root.childs[0])
        } else {
            None
        }
    }
}

impl Default for PolyTree {
    fn default() -> Self {
        Self::new()
    }
}
