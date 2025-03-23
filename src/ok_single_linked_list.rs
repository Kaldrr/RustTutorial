// T -> generic parameter
// More or less template arguments from C++, more constraints, but easier to work with
// in Rust if we want to use T objects in any way, we need to specify their constraints/traits
// like T: Display, like concepts in C++, but we can only use the methods from concepts
pub struct List<T> {
    head: Link<T>,
}

// Previous Link was just a bad implementation of Option
// Let's just use Option, more idiomatic and it has some really nice methods that we've been doing manually!
type Link<T> = Option<Box<Node<T>>>;

struct Node<T> {
    elem: T,
    next: Link<T>,
}

impl<T> List<T> {
    pub fn new() -> Self {
        List { head: None }
    }

    pub fn push(&mut self, elem: T) {
        // Option::take -> std::mem::replace(opt, None)
        // So, take current value and replace with nothing!
        let new_node = Box::new(Node {
            elem,
            next: self.head.take(),
        });

        self.head = Some(new_node);
    }

    pub fn pop(&mut self) -> Option<T> {
        // match None -> None, Some(x) -> Some(y) is also very common in Rust
        // Map allows us to do it easily, it leavs None as is, but Some can be modified
        // Takes closure as a parameter, closure is anonymous/lambda function from C++ and other langauges
        // |arg| is the argument list, everythign else from outer scope seems to be magically avaliable
        // without specifying how we want to capture it for now
        self.head.take().map(|node| {
            self.head = node.next;
            node.elem
        })
    }

    pub fn peek(&self) -> Option<&T> {
        // We need as_ref here, otherwise map would try to take Option<T> by value, moving out the T!
        // as_ref makes a Option<&T> that we can work with
        // Side note, even though we're having indiractions in code, with &, Box etc. Rust is smart enoguh to make . work most of the time
        // No -> or other cursed things
        self.head.as_ref().map(|node| &node.elem)
    }

    pub fn peek_mut(&mut self) -> Option<&mut T> {
        self.head.as_mut().map(|node| &mut node.elem)
    }

    pub fn into_iter(self) -> IntoIter<T> {
        IntoIter(self)
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        let mut cur_link = self.head.take();
        while let Some(mut boxed_node) = cur_link {
            cur_link = boxed_node.next.take();
        }
    }
}

// Iterators
// Has has 3 types of iterators
// IntoIter - T
// IterMut - &mut T
// Iter - &T
// All iterators have only 1 method, next which joins checking if there is a next item + returning it
// compared to C++ derference */->, and checking with other iterator, often an empty/dummy one
// C#/Java MoveNext/HasNext/Current
// Python raising exception on iteration end
// Looks pretty good

// Tuple structs are an alternative form of struct,
// useful for trivial wrappers around other types.
// Also fun fact, Rust doesn't need declarations/definitions in file to be top-down like in C/C++
// into_iter that uses IntoIter lives above but Rust sees it anyway
pub struct IntoIter<T>(List<T>);

// Into Iter consumes the List as it iterates over it
impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        // access fields of a tuple struct numerically
        self.0.pop()
    }
}

// Iter doesn't consume the List, only iterates over it
// Harder as we need to track lifetime of everything now
// The 'a template/generic paremeter is lifetime specifier
// Lifetime needed becuase we have reference inside a struct
// We need compiler to track how long it needs to live
pub struct Iter<'a, T> {
    next: Option<&'a Node<T>>,
}

// Multiple imple  blocks for single struct allowed as well
impl<T> List<T> {
    // No explicit lifetime here, due to lifetime elision this is equivalent to
    // fn iter<'a>(&'a self) -> Iter<'a, T>
    pub fn iter(&self) -> Iter<T> {
        Iter {
            // Equivalent to map(|node| &**node)
            // Or self.next = node.next.as_ref().map::<&Node<T>, _>(|node| &node);
            // ::<> is called turbofish, which is somehow worse syntax then C++ has
            next: self.head.as_deref(),
        }
    }
}

// Lifetime needed here
impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|node| {
            // Equivalent to map(|node| &mut**node)
            self.next = node.next.as_deref();
            &node.elem
        })
    }
}

// In general IterMut is way harder then IntoIter/Iter
// because shared/mutable references only allow one reference to exist
pub struct IterMut<'a, T> {
    next: Option<&'a mut Node<T>>,
}

impl<T> List<T> {
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            next: self.head.as_deref_mut(),
        }
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        // We need to juggle the shared/mutable reference around
        // With things like take
        self.next.take().map(|node| {
            self.next = node.next.as_deref_mut();
            &mut node.elem
        })
    }
}

#[cfg(test)]
mod test {
    use super::List;

    #[test]
    fn basics() {
        // We don't need to write List<i32>
        // The compiler infers that for us on the 1st usage
        // Not on construction like in C++
        let mut list = List::new();

        // Check empty list behaves right
        assert_eq!(list.pop(), None);

        // Populate list
        list.push(1);
        list.push(2);
        list.push(3);

        // Check normal removal
        assert_eq!(list.pop(), Some(3));
        assert_eq!(list.pop(), Some(2));

        // Push some more just to make sure nothing's corrupted
        list.push(4);
        list.push(5);

        // Check normal removal
        assert_eq!(list.pop(), Some(5));
        assert_eq!(list.pop(), Some(4));

        // Check exhaustion
        assert_eq!(list.pop(), Some(1));
        assert_eq!(list.pop(), None);
    }

    #[test]
    fn peek() {
        let mut list = List::new();
        assert_eq!(list.peek(), None);
        assert_eq!(list.peek_mut(), None);
        list.push(1);
        list.push(2);
        list.push(3);

        assert_eq!(list.peek(), Some(&3));
        assert_eq!(list.peek_mut(), Some(&mut 3));
        // In closures, matching argument is a bit different
        // Here value matches to &mut i32, we did not have to write that it is a ref or mutable
        // if we were to wrtie &mut value it would specify that the argument is &mut, but we want to copy it, discard that &mut
        // getting just i32 as a result
        list.peek_mut().map(|value| *value = 42);

        assert_eq!(list.peek(), Some(&42));
        assert_eq!(list.pop(), Some(42));
    }

    #[test]
    fn into_iter() {
        let mut list = List::new();
        list.push(1);
        list.push(2);
        list.push(3);

        let mut iter = list.into_iter();
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter() {
        let mut list = List::new();
        list.push(1);
        list.push(2);
        list.push(3);

        let mut iter = list.iter();
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&1));
    }

    #[test]
    fn iter_mut() {
        let mut list = List::new();
        list.push(1);
        list.push(2);
        list.push(3);

        let mut iter = list.iter_mut();
        assert_eq!(iter.next(), Some(&mut 3));
        assert_eq!(iter.next(), Some(&mut 2));
        assert_eq!(iter.next(), Some(&mut 1));
    }
}
