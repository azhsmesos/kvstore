use std::{
    ptr,
    sync::atomic::{AtomicPtr, AtomicUsize, Ordering},
};

type NodePtr<T> = AtomicPtr<Node<T>>;

struct Node<T> {
    pub item: Option<T>,
    pub next: NodePtr<T>,
}

impl<T> Node<T> {
    pub fn new(item: T) -> Self {
        Self {
            item: Some(item),
            next: AtomicPtr::new(ptr::null_mut()),
        }
    }
    pub fn new_empty() -> Self {
        Self {
            item: None,
            next: AtomicPtr::new(ptr::null_mut()),
        }
    }
}

/// WARNING:
/// LinkedQueue does not fix ABA problem and UAF bug in multi-consumer scenarios
pub struct LinkedQueue<T> {
    // empty list, which is much more easier to implement
    len: AtomicUsize,
    head: NodePtr<T>,
    tail: NodePtr<T>,
}

impl<T> Default for LinkedQueue<T> {
    fn default() -> Self {
        let header = Box::new(Node::new_empty());
        let head = AtomicPtr::from(Box::into_raw(header));
        let tail = AtomicPtr::new(head.load(Ordering::SeqCst));
        Self {
            len: AtomicUsize::new(0),
            head,
            tail,
        }
    }
}

impl<T> LinkedQueue<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.len
            .compare_exchange(0, 0, Ordering::Release, Ordering::Relaxed)
            .is_ok()
    }

    pub fn push(&self, item: T) {
        let new_node = Box::new(Node::new(item));
        let node_ptr: *mut Node<T> = Box::into_raw(new_node);

        let old_tail = self.tail.load(Ordering::Acquire);
        unsafe {
            let mut tail_next = &(*old_tail).next;
            while tail_next
                .compare_exchange(
                    ptr::null_mut(),
                    node_ptr,
                    Ordering::Release,
                    Ordering::Relaxed,
                )
                .is_err()
            {
                let mut tail = tail_next.load(Ordering::Acquire);

                // step to tail
                loop {
                    let nxt = (*tail).next.load(Ordering::Acquire);
                    if nxt.is_null() {
                        break;
                    }
                    tail = nxt;
                }

                tail_next = &(*tail).next;
            }
        }
        let _ =
            self.tail
                .compare_exchange(old_tail, node_ptr, Ordering::Release, Ordering::Relaxed);
        // finish insert, increase length;
        self.len.fetch_add(1, Ordering::SeqCst);
    }

    pub fn pop(&self) -> Option<T> {
        let mut data = None;
        if self.is_empty() {
            return data;
        }
        unsafe {
            let mut head;
            loop {
                head = self.head.load(Ordering::Acquire);
                let next = (*head).next.load(Ordering::Acquire);

                if next.is_null() {
                    return None;
                }

                if self
                    .head
                    .compare_exchange(head, next, Ordering::Release, Ordering::Relaxed)
                    .is_ok()
                {
                    data = (*next).item.take();
                    break;
                }
            }
            // drop `head`
            let _ = Box::from_raw(head);
        };
        self.len.fetch_sub(1, Ordering::SeqCst);

        data
    }
}

impl<T> Drop for LinkedQueue<T> {
    fn drop(&mut self) {
        while self.pop().is_some() {}
        let h = self.head.load(Ordering::SeqCst);
        unsafe {
            // drop `h`
            Box::from_raw(h);
        }
    }
}

#[cfg(test)]
mod lockfree_queue_test {
    use std::{
        sync::{
            atomic::{AtomicI32, Ordering},
            Arc, Barrier,
        },
        thread,
    };
    use crate::entry::Entry;
    use crate::lockfree_queue::LinkedQueue;


    #[test]
    fn test_single() {
        let q = LinkedQueue::new();
        let entry = Entry::new(1, 2);
        q.push(entry);
        println!("{}", q.pop().unwrap().key);
    }
}