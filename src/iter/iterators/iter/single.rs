use super::{
    AbstractMut, Chunk1, ChunkExact1, CurrentId, IntoAbstract, Shiperator, Tight1, Update1,
};
use crate::EntityId;

pub enum Iter1<T: IntoAbstract> {
    Tight(Tight1<T>),
    Update(Update1<T>),
}

impl<T: IntoAbstract> Iter1<T> {
    pub fn into_chunk(self, step: usize) -> Result<Chunk1<T>, Self> {
        match self {
            Self::Tight(tight) => Ok(tight.into_chunk(step)),
            _ => Err(self),
        }
    }
    pub fn into_chunk_exact(self, step: usize) -> Result<ChunkExact1<T>, Self> {
        match self {
            Self::Tight(tight) => Ok(tight.into_chunk_exact(step)),
            _ => Err(self),
        }
    }
}

impl<T: IntoAbstract> Shiperator for Iter1<T> {
    type Item = <T::AbsView as AbstractMut>::Out;

    unsafe fn first_pass(&mut self) -> Option<Self::Item> {
        match self {
            Self::Tight(tight) => tight.first_pass(),
            Self::Update(update) => update.first_pass(),
        }
    }
    unsafe fn post_process(&mut self, item: Self::Item) -> Self::Item {
        match self {
            Self::Tight(tight) => tight.post_process(item),
            Self::Update(update) => update.post_process(item),
        }
    }
}

impl<T: IntoAbstract> CurrentId for Iter1<T> {
    type Id = EntityId;

    unsafe fn current_id(&self) -> Self::Id {
        match self {
            Self::Tight(tight) => tight.current_id(),
            Self::Update(update) => update.current_id(),
        }
    }
}
