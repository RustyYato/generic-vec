use core::ops::{AddAssign, Deref, DerefMut, SubAssign};

pub struct SetLenOnDrop<'a> {
    curr_len: usize,
    len: &'a mut usize,
}

impl<'a> SetLenOnDrop<'a> {
    pub fn new(len: &'a mut usize) -> Self { Self { curr_len: *len, len } }
}

impl Drop for SetLenOnDrop<'_> {
    fn drop(&mut self) { *self.len = self.curr_len; }
}

impl AddAssign<usize> for SetLenOnDrop<'_> {
    fn add_assign(&mut self, rhs: usize) { self.curr_len += rhs; }
}

impl SubAssign<usize> for SetLenOnDrop<'_> {
    fn sub_assign(&mut self, rhs: usize) { self.curr_len -= rhs; }
}

impl Deref for SetLenOnDrop<'_> {
    type Target = usize;

    fn deref(&self) -> &usize { &self.curr_len }
}

impl DerefMut for SetLenOnDrop<'_> {
    fn deref_mut(&mut self) -> &mut usize { &mut self.curr_len }
}
