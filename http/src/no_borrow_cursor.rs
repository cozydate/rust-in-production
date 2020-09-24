pub struct NoBorrowCursor(usize);

impl NoBorrowCursor {
    pub fn new() -> NoBorrowCursor { NoBorrowCursor(0) }

    pub fn split<'a, 'b, 'c>(&mut self, data: &'b [u8], sep: &'c [u8]) -> Option<(&'b [u8], &'b [u8])> {
        if self.0 > data.len() {
            panic!("data is smaller than last call");
        }
        let start = if self.0 < sep.len() { 0 } else { self.0 - sep.len() };
        let region = &data[start..];
        for (region_index, window) in region.windows(sep.len()).enumerate() {
            if window == sep {
                let data_index = start + region_index;
                return Some((&data[..data_index], &data[data_index + sep.len()..]));
            }
        }
        self.0 = data.len();
        None
    }
}
