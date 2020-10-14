pub struct SplitIterator<'a> {
    data: &'a [u8],
    sep: &'a [u8],
}

impl<'a> Iterator for SplitIterator<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        if self.data.is_empty() {
            return None;
        }
        match self.data.windows(self.sep.len())
            .enumerate()
            .filter(|(_index, window)| *window == self.sep)
            .map(|(index, _window)| index)
            .next() {
            Some(index) => {
                let result = &self.data[..index];
                self.data = &self.data[index + self.sep.len()..];
                Some(result)
            }
            None => {
                let result = self.data;
                self.data = &self.data[0..0];
                Some(result)
            }
        }
    }
}

pub fn split_iterate<'a>(data: &'a [u8], sep: &'a [u8]) -> SplitIterator<'a> {
    if sep.is_empty() {
        panic!("split_iterate called with empty sep");
    }
    SplitIterator { data, sep }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn i2s(i: &mut dyn std::iter::Iterator<Item=&[u8]>) -> String {
        let i_vec: Vec<String> = i.map(|b| crate::escape_ascii(b))
            .collect();
        i_vec.join(",")
    }

    #[test]
    fn data_empty() {
        assert_eq!("", i2s(&mut split_iterate(b"", b"s")));
        assert_eq!("", i2s(&mut split_iterate(b"", b"sep1")));
    }

    #[test]
    #[should_panic]
    fn sep_empty() {
        let _ = split_iterate(b"data1", b"");
    }

    #[test]
    fn sep_not_found() {
        assert_eq!("data1", i2s(&mut split_iterate(b"data1", b"s")));
        assert_eq!("data1", i2s(&mut split_iterate(b"data1", b"sep1")));
        assert_eq!("data1", i2s(&mut split_iterate(b"data1", b"data2")));
        assert_eq!("data1", i2s(&mut split_iterate(b"data1", b"0data")));
        assert_eq!("data1", i2s(&mut split_iterate(b"data1", b"0data1")));
        assert_eq!("data1", i2s(&mut split_iterate(b"data1", b"0d")));
        assert_eq!("data1", i2s(&mut split_iterate(b"data1", b"12")));
    }

    #[test]
    fn sep_found_once() {
        assert_eq!(",bc", i2s(&mut split_iterate(b"abc", b"a")));
        assert_eq!("a,c", i2s(&mut split_iterate(b"abc", b"b")));
        assert_eq!("ab", i2s(&mut split_iterate(b"abc", b"c")));
        assert_eq!(",c", i2s(&mut split_iterate(b"abc", b"ab")));
        assert_eq!("a", i2s(&mut split_iterate(b"abc", b"bc")));
        assert_eq!("", i2s(&mut split_iterate(b"abc", b"abc")));
        assert_eq!("a,bc", i2s(&mut split_iterate(b"abbbc", b"bb")));
    }

    #[test]
    fn sep_found_multiple_times() {
        assert_eq!(",,bb", i2s(&mut split_iterate(b"aabb", b"a")));
        assert_eq!("aa,", i2s(&mut split_iterate(b"aabb", b"b")));
        assert_eq!(",b,b", i2s(&mut split_iterate(b"ababa", b"a")));
        assert_eq!("a,a,a", i2s(&mut split_iterate(b"ababa", b"b")));
        assert_eq!("a,,bc", i2s(&mut split_iterate(b"abbbbbc", b"bb")));
    }

    #[test]
    fn next_called_again() {
        let mut s = split_iterate(b"abc", b"b");
        assert_eq!("a".as_bytes(), s.next().unwrap());
        assert_eq!("c".as_bytes(), s.next().unwrap());
        assert_eq!(None, s.next());
        assert_eq!(None, s.next());
        assert_eq!(None, s.next());
    }
}
