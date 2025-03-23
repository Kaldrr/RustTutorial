use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

pub struct List<T> {
    head: Link<T>,
    tail: Link<T>,
}

// RefCell -> borrow checking, but at runtime
// Basically, ReadWriteMutex but single thread
// Either many readers, or one writer, but not at once
// This is getting pretty complicated compared to just C++ pointers
type Link<T> = Option<Rc<RefCell<Node<T>>>>;

struct Node<T> {
    elem: T,
    next: Link<T>,
    prev: Link<T>,
}

impl<T> Node<T> {
    fn new(elem: T) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Node {
            elem,
            prev: None,
            next: None,
        }))
    }
}
impl<T> List<T> {
    pub fn new() -> Self {
        List {
            head: None,
            tail: None,
        }
    }

    pub fn push_front(&mut self, elem: T) {
        let new_head = Node::new(elem);

        // Check which state we are in currently
        match self.head.take() {
            // We were not empty, set new head, and point previous head at new node
            Some(old_head) => {
                // Borrow mut tries to take unique mutable reference, so it tries to take the write lock
                // IT WILL PANIC IF IT FAILS, panic is more or less exception, it starts unwinding
                // (unless compiled to abort instantly)
                old_head.borrow_mut().prev = Some(new_head.clone());
                new_head.borrow_mut().next = Some(old_head);
                self.head = Some(new_head);
            }
            // We were empty, easy case
            None => {
                self.tail = Some(new_head.clone());
                self.head = Some(new_head);
            }
        }
    }

    pub fn pop_front(&mut self) -> Option<T> {
        self.head.take().map(|old_head| {
            // What state was the head in?
            match old_head.borrow_mut().next.take() {
                // There were more elements, erase the previous one + set as current
                Some(new_head) => {
                    new_head.borrow_mut().prev.take();
                    self.head = Some(new_head);
                }
                // This was the only element, set tail to None as well
                None => {
                    self.tail.take();
                }
            }
            // I think I'd rather get an exception in my face then write code like this
            // Thankfully this is a bad code example
            // Rc::try_unwrap -> if it is only Rc that points at this, move the value out of it
            // ok -> convert Result into Option, as unwrap'ing Result would require Node to implement Debug trait
            // which whould then require T to implement Debug, etc etc
            // unwrap just assums that Option is ok, panics if it is not
            // into_inner gets the value out of RefCell
            Rc::try_unwrap(old_head).ok().unwrap().into_inner().elem
        })
    }

    pub fn peek_front(&self) -> Option<Ref<T>> {
        // Originall code didn't work?
        // instead of node.borrow, RefCell::borrow(node) is needed
        // Anyway, we map the Ref returned by RefCell, so instead of pointing at the entire Node
        // it points ONLY at the T, the value that we care about
        // Kind-of like std::shared_ptr, instead of pointing at original object
        // it can point at something, but share the same controll block
        self.head
            .as_ref()
            .map(|node| Ref::map(RefCell::borrow(node), |node: &Node<T>| &node.elem))
    }

    // The rest of the code, copied from tutorial
    // Generally just symetric, we had front operations, so add back operations
    // That act on tail instaed of front
    // And mutable peek-ing

    pub fn push_back(&mut self, elem: T) {
        let new_tail = Node::new(elem);
        match self.tail.take() {
            Some(old_tail) => {
                old_tail.borrow_mut().next = Some(new_tail.clone());
                new_tail.borrow_mut().prev = Some(old_tail);
                self.tail = Some(new_tail);
            }
            None => {
                self.head = Some(new_tail.clone());
                self.tail = Some(new_tail);
            }
        }
    }

    pub fn pop_back(&mut self) -> Option<T> {
        self.tail.take().map(|old_tail| {
            match old_tail.borrow_mut().prev.take() {
                Some(new_tail) => {
                    new_tail.borrow_mut().next.take();
                    self.tail = Some(new_tail);
                }
                None => {
                    self.head.take();
                }
            }
            Rc::try_unwrap(old_tail).ok().unwrap().into_inner().elem
        })
    }

    pub fn peek_back(&self) -> Option<Ref<T>> {
        self.tail
            .as_ref()
            .map(|node| Ref::map(RefCell::borrow(node), |node| &node.elem))
    }

    pub fn peek_back_mut(&mut self) -> Option<RefMut<T>> {
        self.tail
            .as_ref()
            .map(|node| RefMut::map(node.borrow_mut(), |node| &mut node.elem))
    }

    pub fn peek_front_mut(&mut self) -> Option<RefMut<T>> {
        self.head
            .as_ref()
            .map(|node| RefMut::map(node.borrow_mut(), |node| &mut node.elem))
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        while self.pop_front().is_some() {}
    }
}

// Into iter is easy, just pop values one by one
// Consume the List as we go
// The difference is that the list is now double ended, bi-directional
// So, let's implement the from-end iteration as well!
pub struct IntoIter<T>(List<T>);

impl<T> List<T> {
    pub fn into_iter(self) -> IntoIter<T> {
        IntoIter(self)
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop_front()
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<T> {
        self.0.pop_back()
    }
}

// Non consuming iteration
// A true nightmare to even think about doing
// Because RefCell returns references through borrows, iterating while keeping borrows of previous nodes is a nightmare
// We'd get RefCells, of RefCells, of RefCells, etc etc
// Abandoning RefCells and just going with Rc doesn't save us here
// We'd have to totally expose Node, and just do Ref<Node>, Rc<Node> or something
// So it would be kind of ok for an internal data structure, horrible for anything else :(
// RefCells make sharing data through references, very difficult, as we need to keep all the borrows to keep references alive
// as 2nd node depends on 1st, 3rd on 2nd, 4th on 3rd and so on
// While in PersistentLinkedList we could easily handle out Rc to everything, and share to our hearts content, but struggled with unique ownership
// Here we have fairly easy unique ownership, but references are a Lovecraftian Nightmare from which there is no escape
// We're not even going to try IterMut

// pub struct Iter<'a, T>(Option<Ref<'a, Node<T>>>);

// impl<T> List<T> {
//     pub fn iter(&self) -> Iter<T> {
//         Iter(self.head.as_ref().map(|head| RefCell::borrow(head)))
//     }
// }

// impl<'a, T> Iterator for Iter<'a, T> {
//     type Item = Ref<'a, T>;
//     fn next(&mut self) -> Option<Self::Item> {
// Much sadness here
//         self.0.take().map(|node_ref| {
//             let (next, elem) = Ref::map_split(node_ref, |node| (&node.next, &node.elem));
//             self.0 = if next.is_some() {
//                 Some(Ref::map(next, |next| &**next.as_ref().unwrap()))
//             } else {
//                 None
//             };
//             elem
//         })
//     }
// }

#[cfg(test)]
mod test {
    use super::List;

    #[test]
    fn basics() {
        let mut list = List::new();

        // Check empty list behaves right
        assert_eq!(list.pop_front(), None);

        // Populate list
        list.push_front(1);
        list.push_front(2);
        list.push_front(3);

        // Check normal removal
        assert_eq!(list.pop_front(), Some(3));
        assert_eq!(list.pop_front(), Some(2));

        // Push some more just to make sure nothing's corrupted
        list.push_front(4);
        list.push_front(5);

        // Check normal removal
        assert_eq!(list.pop_front(), Some(5));
        assert_eq!(list.pop_front(), Some(4));

        // Check exhaustion
        assert_eq!(list.pop_front(), Some(1));
        assert_eq!(list.pop_front(), None);

        // ---- back -----

        // Check empty list behaves right
        assert_eq!(list.pop_back(), None);

        // Populate list
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);

        // Check normal removal
        assert_eq!(list.pop_back(), Some(3));
        assert_eq!(list.pop_back(), Some(2));

        // Push some more just to make sure nothing's corrupted
        list.push_back(4);
        list.push_back(5);

        // Check normal removal
        assert_eq!(list.pop_back(), Some(5));
        assert_eq!(list.pop_back(), Some(4));

        // Check exhaustion
        assert_eq!(list.pop_back(), Some(1));
        assert_eq!(list.pop_back(), None);
    }

    #[test]
    fn peek() {
        let mut list = List::new();
        assert!(list.peek_front().is_none());
        assert!(list.peek_back().is_none());
        assert!(list.peek_front_mut().is_none());
        assert!(list.peek_back_mut().is_none());

        list.push_front(1);
        list.push_front(2);
        list.push_front(3);

        assert_eq!(&*list.peek_front().unwrap(), &3);
        assert_eq!(&mut *list.peek_front_mut().unwrap(), &mut 3);
        assert_eq!(&*list.peek_back().unwrap(), &1);
        assert_eq!(&mut *list.peek_back_mut().unwrap(), &mut 1);
    }

    #[test]
    fn into_iter() {
        let mut list = List::new();
        list.push_front(1);
        list.push_front(2);
        list.push_front(3);

        let mut iter = list.into_iter();
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next_back(), Some(1));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next_back(), None);
        assert_eq!(iter.next(), None);
    }
}
