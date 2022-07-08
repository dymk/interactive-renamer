pub trait ModuloNthIterMut: ExactSizeIterator {
    fn nth_mod(&mut self, idx: i32) -> Option<Self::Item>;
}

impl<T: ExactSizeIterator> ModuloNthIterMut for T {
    fn nth_mod(&mut self, idx: i32) -> Option<Self::Item> {
        let n = self.len();
        if n == 0 {
            None
        } else {
            let n = n as i32;
            let idx = ((idx % n + n) % n) as usize;
            self.nth(idx)
        }
    }
}

#[cfg(test)]
mod test1 {
    use super::ModuloNthIterMut;

    #[test]
    fn test_mod_iter() {
        let a = [0, 1, 2, 3, 4];
        assert_eq!(Some(&0), a.iter().nth_mod(0));
        assert_eq!(Some(&1), a.iter().nth_mod(1));
        assert_eq!(Some(&4), a.iter().nth_mod(-1));
        assert_eq!(Some(&3), a.iter().nth_mod(-2));
        assert_eq!(Some(&0), a.iter().nth_mod(5));
        assert_eq!(Some(&0), a.iter().nth_mod(-5));
        assert_eq!(None as Option<&i32>, [].iter().nth_mod(0));
    }
}

pub trait AndThenOrOption<T> {
    fn and_then_or<U, F>(self, f: F, default: Option<U>) -> Option<U>
    where
        F: FnOnce(T) -> Option<U>;
}

impl<T> AndThenOrOption<T> for Option<T> {
    fn and_then_or<U, F>(self, f: F, default: Option<U>) -> Option<U>
    where
        F: FnOnce(T) -> Option<U>,
    {
        match self {
            Some(x) => f(x),
            None => default,
        }
    }
}

#[cfg(test)]
mod test2 {
    use super::AndThenOrOption;

    #[test]
    fn option_apply_works() {
        let _none = None as Option<i32>;
        assert_eq!(Some(1), Some(0).and_then_or(|i| Some(i + 1), Some(-1)));
        assert_eq!(Some(-1), _none.and_then_or(|i| Some(i + 1), Some(-1)));
        assert_eq!(None, _none.and_then_or(|i| Some(i + 1), None));
        assert_eq!(None, Some(5).and_then_or(|_| None, Some(-1)));
    }
}
