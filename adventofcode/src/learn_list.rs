//Trying to learn from https://rust-unofficial.github.io/too-many-lists/third.html
use std::rc::Rc;

// Option because it could be the tail and Rc because we need reference
// counted heap-allocated nodes to build a sharable immutable list.
#[allow(dead_code)]
type Link<T> = Option<Rc<Node<T>>>;

// This get allocated on the heap
#[allow(dead_code)]
struct Node<T> {
    data: T,
    next: Link<T>
}

// This is the public piece, which is allocated on the stack (typically).
#[allow(dead_code)]
pub struct List<T> {
    head: Link<T>,
}

impl<T> List<T> {
    #[allow(dead_code)]
    pub fn new() -> Self {
        List{ head: None }
    }

    #[allow(dead_code)]
    pub fn prepend(&self, item: T) -> Self {
        let new_node: Node<T> = Node{data: item, next: self.head.clone()};
        let head: Option<Rc<Node<T>>> = Some(Rc::new(new_node));
        List{head}
    }

    // How THEY wrote it
    fn _head(&self) -> Option<&T> {
        self.head.as_ref().map(|x| &x.data)
    }

    // What I can understand
    #[allow(dead_code)]
    pub fn head(&self) -> Option<&T> {
        match &self.head {
            None => None,
            Some(head_rc) => {
                Some(&head_rc.as_ref().data)
            }
        }
    }

    // How THEY wrote it
    fn _tail(&self) -> Self {
        List{head: self.head.as_ref().and_then(|node| node.next.clone())}
    }

    // What I can understand
    #[allow(dead_code)]
    pub fn tail(&self) -> Self {
        let head: Link<T> = match &self.head {
            None => None,
            Some(head_rc) => {
                head_rc.next.clone()
            },
        };
        List{head}
    }
}


#[cfg(test)]
mod test {
    use super::List;

    #[test]
    fn various() {
        let list1: List<String> = List::new();
        let list2: List<String> = list1.prepend("last".to_string());
        let list3: List<String> = list2.prepend("middle".to_string());
        let list4: List<String> = list3.prepend("first".to_string());
        let list5: List<String> = list3.prepend("other_first".to_string());

        assert_eq!(list4.head(), Some(&"first".to_string()));
        assert_eq!(list4.tail().head(), list5.tail().head());
    }
}
