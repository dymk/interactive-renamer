use std::iter::repeat;

use itertools::Itertools;

pub fn file_name(path: &str) -> &str {
    if let Some(sep) = path.rfind('/') {
        &path[sep + 1..]
    } else {
        path
    }
}

pub fn dir_name(path: &str) -> &str {
    if let Some(sep) = path.rfind('/') {
        &path[..sep]
    } else {
        ""
    }
}

pub fn split_ext(path: &str) -> (&str, Option<&str>) {
    if let Some(sep) = path.rfind('.') {
        (&path[0..sep], Some(&path[sep + 1..]))
    } else {
        (path, None)
    }
}

pub fn join_path(mut a: &str, b: &str) -> String {
    if let Some(sep) = a.rfind('/') {
        if sep == a.len() - 1 {
            a = &a[..a.len() - 1];
        }
    }

    format!("{}/{}", a, b)
}

pub fn compute_prefix_raw(a: &str, b: &str) -> String {
    let a_components = a.split("/").filter(|s| s.len() > 0);
    let b_components = b.split("/").filter(|s| s.len() > 0);

    let prefix_count = a_components
        .clone()
        .zip(b_components.clone())
        .take_while(|(x, y)| x == y)
        .count();

    let remaining_b = b_components.count() - prefix_count;

    let b_back = repeat("..").take(remaining_b);
    let a_forward = a_components.skip(prefix_count);
    b_back.chain(a_forward).join("/")
}

pub fn compute_prefix(a: &str, b: &str) -> String {
    let ret = compute_prefix_raw(a, b);
    return if ret == "" { "./".into() } else { ret + "/" };
}

#[cfg(test)]
mod test {
    use super::{compute_prefix_raw, join_path, split_ext};

    #[test]
    fn split_ext_works() {
        assert_eq!(("foo", Some("bar")), split_ext("foo.bar"));
        assert_eq!(("foo", None), split_ext("foo"));
    }

    #[test]
    fn join_path_works() {
        assert_eq!("foo/bar", join_path("foo", "bar"));
        assert_eq!("foo/bar", join_path("foo/", "bar"));
        assert_eq!("foo/", join_path("foo/", ""));
    }

    #[test]
    fn compute_prefix_works() {
        assert_eq!("..", compute_prefix_raw("foo/", "foo/bar"));
        assert_eq!("", compute_prefix_raw("foo/", "foo/"));
        assert_eq!("../baz", compute_prefix_raw("foo/baz", "foo/smaz"));
        assert_eq!("../../whaz/baz", compute_prefix_raw("whaz/baz", "foo/smaz"));
        assert_eq!("../../whaz/baz", compute_prefix_raw("whaz/baz", "foo/smaz"));
    }
}
