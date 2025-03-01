#[macro_export]
macro_rules! set {
    () => (
        std::collections::BTreeSet::new()
    );
    ($elem:expr; $n:expr) => (
        vec![$elem, $n].into_iter().collect::<std::collections::BTreeSet<_>>()
    );
    ($($x:expr),+ $(,)?) => (
        vec![$($x),+].into_iter().collect::<std::collections::BTreeSet<_>>()
    );
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    #[test]
    fn test_empty_set() {
        assert_eq!(set![], BTreeSet::<()>::new());
    }

    #[test]
    fn test_one_element_set() {
        assert_eq!(set![2], vec![2].into_iter().collect::<BTreeSet<_>>());
        assert_ne!(set![2], vec![1].into_iter().collect::<BTreeSet<_>>());
        assert_ne!(set![2], vec![].into_iter().collect::<BTreeSet<_>>());
    }

    #[test]
    fn test_two_element_set() {
        assert_eq!(set![7, 2], vec![7, 2].into_iter().collect::<BTreeSet<_>>());
        let e = set![7, 2];
        assert_eq!(e, vec![7, 2].into_iter().collect::<BTreeSet<_>>());
        assert_eq!(e, vec![2, 7].into_iter().collect::<BTreeSet<_>>());
    }

    #[test]
    fn test_three_element_set() {
        let e = set![7, 2, 15];
        assert_eq!(e, vec![7, 2, 15].into_iter().collect::<BTreeSet<_>>());
        assert_eq!(e, vec![2, 7, 15].into_iter().collect::<BTreeSet<_>>());
    }

    #[test]
    fn test_four_element_set() {
        let e = set![7, 2, 15, -4];
        assert_eq!(e, vec![7, 2, 15, -4].into_iter().collect::<BTreeSet<_>>());
        assert_eq!(e, vec![-4, 2, 7, 15].into_iter().collect::<BTreeSet<_>>());
        assert_ne!(e, vec![-3, 2, 7, 15].into_iter().collect::<BTreeSet<_>>());
    }
}
