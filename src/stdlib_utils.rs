pub(crate) trait ModuloNthIterMut: ExactSizeIterator {
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

/**
 * Extension for Option<T> implementing a combination like `and_then` + `or`
 * Note however that `(_ as Option<T>).and_then(mapping_func).or(or_default) is
 * _not_ equivalent to (_ as Option<T>).and_then_or(mapping_func, or_default).
 *
 * Truth table:
 * | method       | self      | function -> result | default       | output  |
 * | and_then_or  | None      | _ -> None          | None          | None    |
 * | and_then_or  | None      | _ -> None          | Some(z)       | Some(z) |
 * | and_then_or  | None      | _ -> Some(y)       | None          | None    |
 * | and_then_or  | None      | _ -> Some(y)       | Some(z)       | Some(z) |
 * | and_then_or  | Some(x)   | x -> None          | None          | None    |
 * | and_then_or  | Some(x)   | x -> None          | Some(z)       | None    |
 * | and_then_or  | Some(x)   | x -> Some(y)       | None          | Some(y) |
 * | and_then_or  | Some(x)   | x -> Some(y)       | Some(z)       | Some(y) |
 */
pub(crate) trait AndThenOrOption<T> {
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
mod test_and_then_or {
    use super::AndThenOrOption;

    #[test]
    #[rustfmt::skip]
    #[allow(non_snake_case)]
    fn option_apply_works() {
        let (x, y, z) = (1i8, 2i8, 3i8);
        let None_ = None as Option<i8>;

        assert_eq!(None_  .and_then_or(|_| /* -> */None_  , /* or default */None_  ), /* expected */None_  );
        assert_eq!(None_  .and_then_or(|_| /* -> */None_  , /* or default */Some(z)), /* expected */Some(z));
        assert_eq!(None_  .and_then_or(|_| /* -> */Some(y), /* or default */None_  ), /* expected */None_  );
        assert_eq!(None_  .and_then_or(|_| /* -> */Some(y), /* or default */Some(z)), /* expected */Some(z));
        assert_eq!(Some(x).and_then_or(|_| /* -> */None_  , /* or default */None_  ), /* expected */None_  );
        assert_eq!(Some(x).and_then_or(|_| /* -> */None_  , /* or default */Some(z)), /* expected */None_  );
        assert_eq!(Some(x).and_then_or(|_| /* -> */Some(y), /* or default */None_  ), /* expected */Some(y));
        assert_eq!(Some(x).and_then_or(|_| /* -> */Some(y), /* or default */Some(z)), /* expected */Some(y));
    }
}
