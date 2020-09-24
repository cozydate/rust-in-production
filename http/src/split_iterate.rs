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
    SplitIterator { data, sep }
}
