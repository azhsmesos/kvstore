use std::{cell::RefCell, rc::Rc};

type RefNode = Rc<RefCell<Node>>;

#[derive(Debug, Clone)]
pub struct Node {
    key: usize,
    value: i32,
    down: Option<RefNode>,
    next: Option<RefNode>,
}

impl Node {
    pub fn new(key: usize, value: i32) -> Self {
        Self {
            key,
            value,
            down: None,
            next: None,
        }
    }

    pub fn new_ref_node(key: usize, value: i32) -> RefNode {
        Rc::new(RefCell::new(Node::new(key, value)))
    }

    pub fn new_with_node(node: Node) -> RefNode {
        Rc::new(RefCell::new(node))
    }
}

#[derive(Debug, Clone)]
pub struct SkipList {
    head: RefCell<Option<RefNode>>,
    max_level: usize,
}

impl SkipList {
    pub fn new(max_level: usize) -> Self {
        SkipList {
            head: RefCell::new(None),
            max_level: max_level - 1,
        }
    }

    fn random_level(&self) -> usize {
        let mut n = 0;
        while rand::random::<bool>() && n < self.max_level {
            n += 1;
        }
        n
    }

    pub fn insert(&self, key: usize, value: i32) {
        let level = 1 + if !self.head.borrow().is_none() {
            self.random_level()
        } else {
            self.max_level
        };
        println!("level = {}", level);

        let node = Node::new(key, value);
        let mut current_level = self.max_level + 1;

        let h = { self.head.borrow().clone() };
        match h {
            Some(head) => {
                let mut current = head.clone();
                let mut up_node: Option<Rc<RefCell<Node>>> = None;
                loop {
                    let tmp = current.clone();
                    if current_level > level {
                        if tmp.borrow().key < key {
                            if let Some(next_node) = &tmp.borrow().next {
                                if next_node.borrow().key < key {
                                    current = Rc::clone(next_node);
                                } else {
                                    if let Some(down_node) = &tmp.borrow().down {
                                        current = Rc::clone(down_node);
                                        current_level -= 1;
                                    } else {
                                        return;
                                    }
                                }
                            } else {
                                if let Some(down_node) = &tmp.borrow().down {
                                    current = Rc::clone(down_node);
                                    current_level -= 1;
                                } else {
                                    return;
                                }
                            }
                        }
                    } else {
                        let current_key = { tmp.borrow().key };
                        if current_key < key {
                            let next_node = { &tmp.borrow().next.clone() };
                            if let Some(next_node) = next_node {
                                if next_node.borrow().key < key {
                                    current = Rc::clone(next_node);
                                    continue;
                                } else {
                                    let new_node = Node::new_with_node(node.clone());
                                    new_node.borrow_mut().next = Some(Rc::clone(next_node));
                                    current.borrow_mut().next = Some(new_node.clone());
                                    match up_node {
                                        Some(ref up) => {
                                            up.borrow_mut().down = Some(new_node.clone());
                                        }
                                        None => {
                                            up_node = Some(new_node.clone());
                                        }
                                    }
                                    if let Some(down_node) = &tmp.borrow().down {
                                        current = Rc::clone(down_node);
                                        current_level -= 1;
                                        continue;
                                    }
                                    return;
                                }
                            }
                            let new_node = Node::new_with_node(node.clone());
                            current.borrow_mut().next = Some(new_node.clone());
                            match up_node {
                                Some(ref up) => {
                                    up.borrow_mut().down = Some(new_node.clone());
                                }
                                None => {
                                    up_node = Some(new_node.clone());
                                }
                            }
                            if let Some(down_node) = &tmp.borrow().down {
                                current = Rc::clone(down_node);
                                current_level -= 1;
                            } else {
                                return;
                            }
                        }
                    }
                }
            }
            None => {
                let mut current = None;
                for i in 0..level {
                    if i == 0 {
                        let n = Node::new_with_node(node.clone());
                        *self.head.borrow_mut() = Some(n.clone());
                        current = Some(n.clone());
                        continue;
                    }
                    let new_node = Some(Node::new_with_node(node.clone()));
                    match current {
                        Some(ref c) => {
                            c.borrow_mut().down = new_node.clone();
                        }
                        None => {
                            return;
                        }
                    }
                    current = new_node.clone();
                }
            }
        }
    }

    fn find(&self, key: usize) -> Option<i32> {
        let h = { self.head.borrow().clone() };
        match h {
            Some(ref head) => {
                let mut current = head.clone();
                loop {
                    let tmp = current.clone();
                    if tmp.borrow().key == key {
                        return Some(current.borrow().value);
                    }
                    if tmp.borrow().key < key {
                        if let Some(next_node) = &tmp.borrow().next {
                            if next_node.borrow().key <= key {
                                current = Rc::clone(next_node);
                            } else {
                                if let Some(down_node) = &tmp.borrow().down {
                                    current = Rc::clone(down_node);
                                } else {
                                    return None;
                                }
                            }
                        } else {
                            if let Some(down_node) = &tmp.borrow().down {
                                current = Rc::clone(down_node);
                            } else {
                                return None;
                            }
                        }
                    }
                }
            }
            None => None,
        }
    }

    // fn print_level_path(&self) {
    //     match self.head {
    //         Some(ref head) => {
    //             let mut current = head.clone();
    //             loop {
    //                 let tmp = current.clone();
    //                 println!("key = {}, value = {}", tmp.borrow().key, tmp.borrow().value);
    //                 if let Some(value) = &tmp.borrow().down {
    //                     current = Rc::clone(value);
    //                 } else {
    //                     let mut n = tmp.clone();
    //                     loop {
    //                         let n_tmp = n.clone();
    //                         if let Some(value) = &n_tmp.borrow().next {
    //                             println!(
    //                                 "key = {}, value = {}",
    //                                 value.borrow().key,
    //                                 value.borrow().value
    //                             );
    //                             n = Rc::clone(value);
    //                         } else {
    //                             return;
    //                         };
    //                     }
    //                     // return;
    //                 };
    //             }
    //         }
    //         None => {}
    //     }
    // }

    pub fn delete(&self, key: usize) {
        todo!()
    }

    pub fn get_range(&self, key: i32, value: i32) -> Option<Vec<&i32>> {
        todo!()
    }
}

#[cfg(test)]
mod tests {

    use super::SkipList;

    #[test]
    fn test() {
        let skiplist = SkipList::new(2);
        skiplist.insert(1, 1);
        skiplist.insert(3, 3);
        skiplist.insert(2, 2);
        skiplist.insert(4, 4);
        // skiplist.print_level_path();
        assert_eq!(Some(2), skiplist.find(2));
        assert_eq!(Some(3), skiplist.find(3));
    }
}
