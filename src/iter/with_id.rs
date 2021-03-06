use super::{CurrentId, Shiperator};

pub struct WithId<I> {
    iter: I,
}

impl<I> WithId<I> {
    pub(super) fn new(iter: I) -> Self {
        WithId { iter }
    }
}

impl<I: CurrentId> Shiperator for WithId<I> {
    type Item = (I::Id, I::Item);

    unsafe fn first_pass(&mut self) -> Option<Self::Item> {
        let item = self.iter.first_pass()?;
        Some((self.iter.current_id(), item))
    }
    unsafe fn post_process(&mut self, (id, item): Self::Item) -> Self::Item {
        let item = self.iter.post_process(item);
        (id, item)
    }
}

impl<I: CurrentId> CurrentId for WithId<I> {
    type Id = I::Id;

    unsafe fn current_id(&self) -> Self::Id {
        self.iter.current_id()
    }
}
