use std::mem;

pub struct List {
    head: Link,
}

// Enum can act as a union/std::variant, but language supports it way more
// Keeps the active member around, and allows pattern matching :D
// Important optimization: null pointer optimization
// Rust gives special treatment to Box, &, &mut, Rc, Arc, Vec in enums (Option is an Enum as well)
// It knows they can't be empty (0x0000000000000000) as they are never null
// So right now Empty=0x0000000000000000, More(_) anything else, enum tag can be skipped, saving memory
enum Link {
    Empty,
    // Box -> std::unique_ptr that can't be empty/invalid
    // There's no move constructor, when objects get moved around the previous object gets destroyed
    // Objects also don't get notified that they are being moved around
    More(Box<Node>),
}

// Needed because enum internals are always public
// If it would refer to a internal struct in public way, it would be bad :(
struct Node {
    elem: i32,
    next: Link,
}

impl List {
    pub fn new() -> Self {
        List { head: Link::Empty }
    }

    pub fn push(&mut self, elem: i32) {
        // mem::replace -> std::exchange
        // Rust won't allow us to swap out self.head in any other way (for now)
        // exception safety, otherwise if we would panic (and catch it) it would leave struct in invalid state
        // mem::replace is safe function, it just does a bunch of memcpy's, as Rust structs are trivial to move around
        // so it will never panic, and leave everything in a valid state
        let new_node = Box::new(Node {
            elem,
            next: mem::replace(&mut self.head, Link::Empty),
        });

        self.head = Link::More(new_node);
    }

    pub fn pop(&mut self) -> Option<i32> {
        // pattern match'ing can return a value, neat
        // also last statement (if it doesn't end with ;) is the return value
        // so we can skip `return`
        match mem::replace(&mut self.head, Link::Empty) {
            Link::Empty => None,
            Link::More(node) => {
                self.head = node.next;
                Some(node.elem)
            }
        }
    }
}

// We need a custom Drop (C++ destructor equivalent) to avoid stack overflow
// Each item in list will require a new function call
// Tail Recursion can't help us here :(
// So do it iterative
impl Drop for List {
    fn drop(&mut self) {
        // Take 1st element
        let mut cur_link = mem::replace(&mut self.head, Link::Empty);

        // `while let` == "do this thing until this pattern doesn't match"
        while let Link::More(mut boxed_node) = cur_link {
            // We COULD do `pop` in a loop
            // But `pop` moves (through memcpy) the object around
            // If object is big, that is really bad...
            cur_link = mem::replace(&mut boxed_node.next, Link::Empty);
            // boxed_node goes out of scope and gets dropped here;
            // but its Node's `next` field has been set to Link::Empty
            // so no unbounded recursion occurs.
        }
    }
}

// Only for test builds
// built-in support for UT's in the language, thank god
// C++ UT's are too often a Lovecraftian nightmare
// run with `cargo test`
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
}
