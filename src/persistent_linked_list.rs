// We want to achieve something like this
// list1 -> A ---+
//               |
//               v
// list2 ------> B -> C -> D
//               ^
//               |
// list3 -> X ---+
//
// list1/list2/list3 share the B node

use std::rc::Rc;

pub struct List<T> {
    head: Link<T>,
}

// Rc instead of Box
// Rc is std::shared_ptr, but ONLY in current thread
// for multithreading there is Arc which is exactly std::shared_ptr
type Link<T> = Option<Rc<Node<T>>>;

struct Node<T> {
    elem: T,
    next: Link<T>,
}

impl<T> List<T> {
    pub fn new() -> Self {
        List { head: None }
    }

    pub fn prepend(&self, elem: T) -> List<T> {
        // Clone -> C++ copy constructor
        // Cloning a Option clones its contents
        // Cloning a Rc creates a new shared-ptr into what it points into
        // So the new list that we return will have (for now) unique ownership of the head-node
        // But will share the rest of the nodes
        List {
            head: Some(Rc::new(Node {
                elem,
                next: self.head.clone(),
            })),
        }
    }

    pub fn tail(&self) -> List<T> {
        // and_then instead of map
        // map takes a function f(Option(X) -> Y), the map itself returns Option, but the function map takes takes value, and returns value, map wraps the returned value into optional
        // f(X -> Y), map will call the f and wrap it's output into optional
        // and_then only calls the function, the function itself must return optional
        // f(X -> Option(Y))
        // as node.next is Option<Rc<_>>, map would create Option<Option<Rc<Node<T>>>>
        // but and_then returns Option<Rc<Node<T>>>, which is what we want!
        List {
            head: self.head.as_ref().and_then(|node| node.next.clone()),
        }
    }

    pub fn head(&self) -> Option<&T> {
        self.head.as_ref().map(|node| &node.elem)
    }
}

// Different drop, as Rc is a more complex then Box
// We can't just take the value out of it easily
// Drop still needed to avoid stack overflow if the list is too big
impl<T> Drop for List<T> {
    fn drop(&mut self) {
        let mut head = self.head.take();
        while let Some(node) = head {
            // Rc::try_unwrap checks if this RC is the ONLY owner of the value
            if let Ok(mut node) = Rc::try_unwrap(node) {
                head = node.next.take();
            } else {
                break;
            }
        }
    }
}

// We CAN NOT implement IntoInter/IterMut for this type
// Rc<T> is more or less std::shared_ptr<const T>
// impossible to reliably get the value out of it, which IntoIter requries
// or to get a mutable reference, which IterMut requries
// we can only do that when there's EXACTLY 1 reference, with Rc::try_unwrap
// but that would be... weird for iteration, iteration that can fail???
// C++ could do some const_cast magic (bleh), but Rust doesn't have it
// probably for the best, too easy to abuse
// and compiler needs to always remember that const can be casted away if original object isn't const...
pub struct Iter<'a, T> {
    next: Option<&'a Node<T>>,
}

impl<T> List<T> {
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            next: self.head.as_deref(),
        }
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|node| {
            self.next = node.next.as_deref();
            &node.elem
        })
    }
}

#[cfg(test)]
mod test {
    use super::List;

    #[test]
    fn basics() {
        let list = List::new();
        assert_eq!(list.head(), None);

        let list = list.prepend(1).prepend(2).prepend(3);
        assert_eq!(list.head(), Some(&3));

        let list = list.tail();
        assert_eq!(list.head(), Some(&2));

        let list = list.tail();
        assert_eq!(list.head(), Some(&1));

        let list = list.tail();
        assert_eq!(list.head(), None);

        // Make sure empty tail works
        let list = list.tail();
        assert_eq!(list.head(), None);
    }

    #[test]
    fn iter() {
        let list = List::new().prepend(1).prepend(2).prepend(3);

        let mut iter = list.iter();
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&1));
    }
}
