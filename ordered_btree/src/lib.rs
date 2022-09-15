use arrayvec::ArrayVec;

pub struct OrderedBTree {}
pub struct Node<V, const B: usize> {
    edges: ArrayVec<Box<Node<V, B>>, B>,
    values: ArrayVec<V, B>,
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
