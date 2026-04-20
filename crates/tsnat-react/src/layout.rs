// tsnat-react/src/layout.rs
use std::collections::HashMap;
use yoga::{Direction, FlexDirection, Node, StyleUnit};

/// A Layout Node linking the layout geometry to the user's React Widget identity 
pub struct LayoutNode {
    pub widget_id: u32,
    pub yoga_node: Node,
}

/// The LayoutTree maintains the overall Yoga structure. 
pub struct LayoutTree {
    nodes: HashMap<u32, LayoutNode>,
    root_id: Option<u32>,
}

impl LayoutTree {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            root_id: None,
        }
    }

    pub fn set_root(&mut self, widget_id: u32) {
        self.root_id = Some(widget_id);
    }

    pub fn insert_node(&mut self, widget_id: u32) {
        let mut node = Node::new();
        // Give sensible defaults for Flex containers
        node.set_flex_direction(FlexDirection::Column);
        
        // Use default configuration.
        self.nodes.insert(
            widget_id,
            LayoutNode {
                widget_id,
                yoga_node: node,
            },
        );
    }

    pub fn insert_child(&mut self, parent_id: u32, child_id: u32, index: u32) {
        if let Some(mut child) = self.nodes.remove(&child_id) {
            if let Some(parent) = self.nodes.get_mut(&parent_id) {
                parent.yoga_node.insert_child(&mut child.yoga_node, index as usize);
            }
            self.nodes.insert(child_id, child);
        }
    }

    pub fn remove_child(&mut self, parent_id: u32, child_id: u32) {
        let mut child_node = self.nodes.remove(&child_id);
        if let Some(ref mut child) = child_node {
            if let Some(parent) = self.nodes.get_mut(&parent_id) {
                parent.yoga_node.remove_child(&mut child.yoga_node);
            }
        }
        if let Some(child) = child_node {
            self.nodes.insert(child_id, child);
        }
    }

    pub fn calculate_layout(&mut self, width: f32, height: f32) {
        if let Some(root_id) = self.root_id {
            if let Some(root) = self.nodes.get_mut(&root_id) {
                // Pin the root node's width and height to the SDL bounds
                root.yoga_node.set_width(StyleUnit::Point(width.into()));
                root.yoga_node.set_height(StyleUnit::Point(height.into()));
                root.yoga_node.calculate_layout(width, height, Direction::LTR);
            }
        }
    }

    pub fn get_layout(&self, widget_id: u32) -> Option<(f32, f32, f32, f32)> {
        self.nodes.get(&widget_id).map(|node| {
            let layout = node.yoga_node.get_layout();
            (
                layout.left(),
                layout.top(),
                layout.width(),
                layout.height()
            )
        })
    }
}
