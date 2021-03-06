use super::{AbstractMut, IntoAbstract, Shiperator};

pub struct ChunkExact1<T: IntoAbstract> {
    pub(crate) data: T::AbsView,
    pub(crate) current: usize,
    pub(crate) end: usize,
    pub(crate) step: usize,
}

impl<T: IntoAbstract> ChunkExact1<T> {
    pub fn remainder(&mut self) -> <T::AbsView as AbstractMut>::Slice {
        let remainder = std::cmp::min(self.end - self.current, self.end % self.step);
        let old_end = self.end;
        self.end -= remainder;
        unsafe { self.data.get_data_slice(self.end..old_end) }
    }
}

impl<T: IntoAbstract> Shiperator for ChunkExact1<T> {
    type Item = <T::AbsView as AbstractMut>::Slice;

    unsafe fn first_pass(&mut self) -> Option<Self::Item> {
        let current = self.current;
        if current + self.step <= self.end {
            self.current += self.step;
            Some(self.data.get_data_slice(current..self.current))
        } else {
            None
        }
    }
    unsafe fn post_process(&mut self, item: Self::Item) -> Self::Item {
        item
    }
}
