use crate::storage::Storage;
use crate::world::World;
use std::any::TypeId;

// Register multiple storages at once
pub trait Register {
    fn register(world: &World);
}

impl Register for () {
    fn register(_: &World) {}
}

macro_rules! impl_register {
    ($(($type: ident, $index: tt))+) => {
        impl<$($type: 'static + Send + Sync),+> Register for ($($type,)+) {
            fn register(world: &World) {
                let mut all_storages = world.storages.try_borrow_mut().unwrap();
                $({
                    let type_id = TypeId::of::<$type>();
                    all_storages.0.entry(type_id).or_insert_with(|| {
                        Storage::new::<$type>()
                    });
                })+
            }
        }
    }
}

macro_rules! register {
    ($(($type: ident, $index: tt))*;($type1: ident, $index1: tt) $(($queue_type: ident, $queue_index: tt))*) => {
        impl_register![$(($type, $index))*];
        register![$(($type, $index))* ($type1, $index1); $(($queue_type, $queue_index))*];
    };
    ($(($type: ident, $index: tt))*;) => {
        impl_register![$(($type, $index))*];
    }
}

register![(A, 0); (B, 1) (C, 2) (D, 3) (E, 4) (F, 5) (G, 6) (H, 7) (I, 8) (J, 9)];
