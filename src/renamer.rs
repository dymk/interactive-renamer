use regex::Regex;

pub struct Renamer<'a> {
    finder: &'a str,
    replacer: &'a str,
}

impl<'a> Renamer<'a> {
    pub fn new(finder: &'a str, replacer: &'a str) -> Renamer<'a> {
        Renamer { finder, replacer }
    }

    pub fn process(&self, input: &str) -> String {
        let finder_regex = Regex::new(self.finder).unwrap();
        let captures = match finder_regex.captures(&input) {
            Some(cap) => cap,
            None => return self.replacer.to_string().clone(),
        };

        let mut replaced = self.replacer.to_string();

        let mut idx = 1;
        loop {
            let pat = format!("${}", idx);
            let cap = match captures.get(idx) {
                Some(cap) => cap,
                None => return replaced,
            };

            if replaced.contains(pat.as_str()) {
                replaced = replaced.replace(pat.as_str(), cap.as_str());
            } else {
                return replaced;
            }

            idx += 1;
        }
    }
}

#[cfg(test)]
mod test {
    use crate::renamer::Renamer;

    #[test]
    fn works() {
        let r = Renamer::new("(.+)", "$1");
        assert_eq!("foo", r.process("foo"));
        assert_eq!("asd23$1", r.process("asd23$1"));

        let r = Renamer::new("(.+)", "$1_asdf");
        assert_eq!("foo_asdf", r.process("foo"));
        assert_eq!("asd23$1_asdf", r.process("asd23$1"));

        let r = Renamer::new("foo(\\d+)", "$1_foo");
        assert_eq!("1_foo", r.process("foo1"));
        assert_eq!("345_foo", r.process("foo345"));
        assert_eq!("$1_foo", r.process("foo"));
        assert_eq!("$1_foo", r.process("1234"));
    }
}
