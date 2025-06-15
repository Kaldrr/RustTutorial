// We'll try to implement a queue
// We will have to cache the first/last node
// To make this work, we will pop from front, and push to the end
// Otherwise, poping from end would be harder, as we need to have access to node one before end
// We could have cached it I guess?

use std::ptr;

// C++ with extra steps
// We're full on pointers in here
// Pointers and references have slightly arcane rules in Rust
// Related to aliasing, and the fact only 1 reference should be able to write to an object
// If we mix writing through pointers and references, without keeping an order
// Rust will be quite upset, to avoid this for now, going full on pointers
// will be ok enoguh
// Implementation of the class itself isn't really THAT much safer then what we could achieve in C++
// But every usage of it will be, nasty parts are restricted to unsafe code blocks
pub struct List<T> {
    head: Link<T>,
    // C/C++ raw pointer
    // No lifetime
    // Can be null (no null-pointer-optimization on Option :( )
    // Can be misaligned
    // Can be dangling
    // Can point to uninitialized memory
    // Can cast mutability out of it, if we're wrong about it, we're gonna have a bad time
    // Can be cast to/from integer type
    // So the same as in C/C++ generally
    // *const T -> C/C++ const T*
    // *mut T -> C/C++ T*
    tail: *mut Node<T>,
}

type Link<T> = *mut Node<T>;

struct Node<T> {
    elem: T,
    next: Link<T>,
}

impl<T> List<T> {
    pub fn new() -> Self {
        List {
            head: ptr::null_mut(),
            tail: ptr::null_mut(),
        }
    }

    pub fn push(&mut self, elem: T) {
        // std::unique_ptr::release
        // Return the raw pointer to allocatd thing
        let new_tail = Box::into_raw(Box::new(Node {
            elem,
            next: ptr::null_mut(),
        }));

        // .is_null checks for null, equivalent to checking for None
        if !self.tail.is_null() {
            // Dereferencing pointer is unsafe in Rust
            // So when we have memory issues, we know where to look, in unsafe code
            // We have 2 choices here
            // 1. Make entire function unsafe
            // not great, we want the Queue and it's functions to be safe :(
            // 2. Unsafe block
            // make only a block unsafe
            unsafe {
                // If the old tail existed, update it to point to the new tail
                (*self.tail).next = new_tail;
            }
        } else {
            // Otherwise, update the head to point to it
            self.head = new_tail;
        }

        self.tail = new_tail;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.head.is_null() {
            None
        } else {
            unsafe {
                // std::unique_ptr{pointer}
                // Adopt existing alocation into a Box that will free it
                let node = Box::from_raw(self.head);
                self.head = (*node).next;

                if self.head.is_null() {
                    self.tail = ptr::null_mut();
                }

                Some(node.elem)
            }
        }
    }

    pub fn peek(&self) -> Option<&T> {
        unsafe { self.head.as_ref().map(|node| &node.elem) }
    }

    pub fn peek_mut(&mut self) -> Option<&mut T> {
        unsafe { self.head.as_mut().map(|node| &mut node.elem) }
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        while !self.head.is_null() {
            unsafe {
                let next = (*self.head).next;
                drop(Box::from_raw(std::mem::replace(&mut self.head, next)));
            }
        }
    }
}

pub struct IntoIter<T>(List<T>);

pub struct Iter<'a, T> {
    next: Option<&'a Node<T>>,
}

pub struct IterMut<'a, T> {
    next: Option<&'a mut Node<T>>,
}

impl<T> List<T> {
    pub fn into_iter(self) -> IntoIter<T> {
        IntoIter(self)
    }

    pub fn iter(&self) -> Iter<'_, T> {
        unsafe {
            Iter {
                // DANGER: unbound lifetime
                // as_ref<'a>(self) -> Option<&'a T>
                // The lifetime of the returned &T is not bound to ANYTHING
                // We need to place it somewhere that is bounded as soon as possible
                // usually, return from function
                next: self.head.as_ref(),
            }
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        unsafe {
            IterMut {
                next: self.head.as_mut(),
            }
        }
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop()
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            self.next.map(|node| {
                self.next = node.next.as_ref();
                &node.elem
            })
        }
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            self.next.take().map(|node| {
                self.next = node.next.as_mut();
                &mut node.elem
            })
        }
    }
}

#[cfg(test)]
mod test {
    use super::List;
    #[test]
    fn basics() {
        let mut list = List::new();

        // Check empty list behaves right
        assert_eq!(list.pop(), None);

        // Populate list
        list.push(1);
        list.push(2);
        list.push(3);

        // Check normal removal
        assert_eq!(list.pop(), Some(1));
        assert_eq!(list.pop(), Some(2));

        // Push some more just to make sure nothing's corrupted
        list.push(4);
        list.push(5);

        // Check normal removal
        assert_eq!(list.pop(), Some(3));
        assert_eq!(list.pop(), Some(4));

        // Check exhaustion
        assert_eq!(list.pop(), Some(5));
        assert_eq!(list.pop(), None);

        // Check the exhaustion case fixed the pointer right
        list.push(6);
        list.push(7);

        // Check normal removal
        assert_eq!(list.pop(), Some(6));
        assert_eq!(list.pop(), Some(7));
        assert_eq!(list.pop(), None);
    }

    #[test]
    fn into_iter() {
        let mut list = List::new();
        list.push(1);
        list.push(2);
        list.push(3);

        let mut iter = list.into_iter();
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter() {
        let mut list = List::new();
        list.push(1);
        list.push(2);
        list.push(3);

        let mut iter = list.iter();
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter_mut() {
        let mut list = List::new();
        list.push(1);
        list.push(2);
        list.push(3);

        let mut iter = list.iter_mut();
        assert_eq!(iter.next(), Some(&mut 1));
        assert_eq!(iter.next(), Some(&mut 2));
        assert_eq!(iter.next(), Some(&mut 3));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn miri_food() {
        let mut list = List::new();

        list.push(1);
        list.push(2);
        list.push(3);

        assert!(list.pop() == Some(1));
        list.push(4);
        assert!(list.pop() == Some(2));
        list.push(5);

        assert!(list.peek() == Some(&3));
        list.push(6);
        list.peek_mut().map(|x| *x *= 10);
        assert!(list.peek() == Some(&30));
        assert!(list.pop() == Some(30));

        for elem in list.iter_mut() {
            *elem *= 100;
        }

        let mut iter = list.iter();
        assert_eq!(iter.next(), Some(&400));
        assert_eq!(iter.next(), Some(&500));
        assert_eq!(iter.next(), Some(&600));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);

        assert!(list.pop() == Some(400));
        list.peek_mut().map(|x| *x *= 10);
        assert!(list.peek() == Some(&5000));
        list.push(7);

        // Drop it on the ground and let the dtor exercise itself
    }
}
