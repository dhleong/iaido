use std::{collections::HashMap, ops, rc::Rc};

use crate::input::Key;

use super::KeyHandler;

#[derive(Clone)]
pub struct KeyTreeNode {
    pub children: HashMap<Key, KeyTreeNode>,
    handler: Option<Rc<KeyHandler>>,
    handler_override: Option<Rc<KeyHandler>>,
}

impl KeyTreeNode {
    pub fn root() -> Self {
        Self {
            children: HashMap::new(),
            handler: None,
            handler_override: None,
        }
    }

    pub fn get_handler(&self) -> Option<&Rc<KeyHandler>> {
        if let Some(overridden) = &self.handler_override {
            Some(overridden)
        } else if let Some(handler) = &self.handler {
            Some(handler)
        } else {
            None
        }
    }

    pub fn insert(&mut self, keys: &[Key], handler: Box<KeyHandler>) {
        if keys.is_empty() {
            self.handler = Some(Rc::new(handler));
        } else {
            let first_key = keys[0];
            let node = self
                .children
                .entry(first_key)
                .or_insert(KeyTreeNode::root());
            node.insert(&keys[1..], handler);
        }
    }
}

impl ops::Add<KeyTreeNode> for KeyTreeNode {
    type Output = KeyTreeNode;

    fn add(self, mut rhs: KeyTreeNode) -> Self::Output {
        let mut result = KeyTreeNode::root();

        if let Some(rhs_handler) = rhs.handler {
            result.handler = Some(rhs_handler);
        } else {
            result.handler = self.handler;
        }

        // combine shared child nodes and insert our unmatched nodes
        for (key, child) in self.children {
            if let Some(rhs_child) = rhs.children.remove(&key) {
                result.children.insert(key, child + rhs_child);
            } else {
                result.children.insert(key, child);
            }
        }

        // insert their unmatched nodes
        for (key, child) in rhs.children {
            if !result.children.contains_key(&key) {
                result.children.insert(key, child);
            }
        }

        result
    }
}
