use std::any::TypeId;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

/// AtomicRefCell's borrow error.
///
/// Unique means the BorrowState was mutably borrowed when an illegal borrow occured.
///
/// Shared means the BorrowState was immutably borrowed when an illegal borrow occured.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Borrow {
    Unique,
    Shared,
}

impl Error for Borrow {}

impl Debug for Borrow {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Borrow::Unique => fmt.write_str("Cannot mutably borrow while already borrowed."),
            Borrow::Shared => {
                fmt.write_str("Cannot immutably borrow while already mutably borrowed.")
            }
        }
    }
}

impl Display for Borrow {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        Debug::fmt(self, fmt)
    }
}

/// Error related to acquiring a storage.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GetStorage {
    AllStoragesBorrow(Borrow),
    StorageBorrow((&'static str, Borrow)),
    MissingComponent(&'static str),
    NonUnique((&'static str, Borrow)),
    MissingUnique(&'static str),
    Entities(Borrow),
}

impl Error for GetStorage {}

impl Debug for GetStorage {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        match self {
            GetStorage::AllStoragesBorrow(borrow) => match borrow {
                Borrow::Unique => fmt.write_str("Cannot mutably borrow AllStorages while it's already borrowed (AllStorages is borrowed to access any storage)."),
                Borrow::Shared => {
                    fmt.write_str("Cannot immutably borrow AllStorages while it's already mutably borrowed.")
                }
            },
            GetStorage::StorageBorrow((name, borrow)) => match borrow {
                Borrow::Unique => fmt.write_fmt(format_args!("Cannot mutably borrow {:?} storage while it's already borrowed.", name)),
                Borrow::Shared => {
                    fmt.write_fmt(format_args!("Cannot immutably borrow {:?} storage while it's already mutably borrowed.", name))
                }
            },
            GetStorage::MissingComponent(name) => fmt.write_fmt(format_args!("No storage exists for {name}.\nConsider adding this line after the creation of World: world.register::<{name}>();", name = name)),
            GetStorage::MissingUnique(name) => fmt.write_fmt(format_args!("No unique storage exists for {name}.\nConsider adding this line after the creation of World: world.register_unique::<{name}>(/* your_storage */);", name = name)),
            GetStorage::NonUnique((name, mutation)) => match mutation {
                Borrow::Shared => fmt.write_fmt(format_args!("{name}'s storage isn't unique.\nYou might have forgotten to declare it, replace world.register::<{name}>() by world.register_unique(/* your_storage */).\nIf it isn't supposed to be a unique storage, replace Unique<&{name}> by &{name}.", name = name)),
                Borrow::Unique => fmt.write_fmt(format_args!("{name}'s storage isn't unique.\nYou might have forgotten to declare it, replace world.register::<{name}>() by world.register_unique(/* your_storage */).\nIf it isn't supposed to be a unique storage, replace Unique<&mut {name}> by &mut {name}.", name = name)),
            },
            GetStorage::Entities(borrow) => match borrow {
                Borrow::Unique => fmt.write_str("Cannot mutably borrow Entities storage while it's already borrowed."),
                Borrow::Shared => {
                    fmt.write_str("Cannot immutably borrow Entities storage while it's already mutably borrowed.")
                }
            },
        }
    }
}

impl Display for GetStorage {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        Debug::fmt(self, fmt)
    }
}

/// Error related to adding an entity.
///
/// AllStoragesBorrow means an add_storage operation is in progress.
///
/// Entities means entities is already borrowed.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum NewEntity {
    AllStoragesBorrow(Borrow),
    Entities(Borrow),
}

impl Error for NewEntity {}

impl Debug for NewEntity {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        match self {
            NewEntity::AllStoragesBorrow(borrow) => match borrow {
                Borrow::Unique => fmt.write_str("Cannot mutably borrow all storages while it's already borrowed (this include component storage)."),
                Borrow::Shared => {
                    fmt.write_str("Cannot immutably borrow all storages while it's already mutably borrowed.")
                }
            },
            NewEntity::Entities(borrow) => match borrow {
                Borrow::Unique => fmt.write_str("Cannot mutably borrow entities while it's already borrowed."),
                Borrow::Shared => unreachable!(),
            },
        }
    }
}

impl Display for NewEntity {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        Debug::fmt(self, fmt)
    }
}

/// If a storage is packed_owned all storages packed with it have to be
/// passed in the add_component call even if no components are added.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AddComponent {
    // `TypeId` of the storage requirering more storages
    MissingPackStorage(TypeId),
    EntityIsNotAlive,
}

impl Error for AddComponent {}

impl Debug for AddComponent {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        match self {
            AddComponent::MissingPackStorage(type_id) => fmt.write_fmt(format_args!("Missing storage for type ({:?}). To add a packed component you have to pass all storages packed with it. Even if you just add one component.", type_id)),
            AddComponent::EntityIsNotAlive => fmt.write_str("Entity has to be alive to add component to it."),
        }
    }
}

impl Display for AddComponent {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        Debug::fmt(self, fmt)
    }
}

/// Error occuring when a pack can't be made.
/// It could be a borrow issue or one of the storage could already have
/// an incompatible pack or the storage could be unique.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Pack {
    GetStorage(GetStorage),
    AlreadyTightPack(TypeId),
    AlreadyLoosePack(TypeId),
    AlreadyUpdatePack(TypeId),
    UniqueStorage(&'static str),
}

impl Error for Pack {}

impl From<GetStorage> for Pack {
    fn from(get_storage: GetStorage) -> Self {
        Pack::GetStorage(get_storage)
    }
}

impl Debug for Pack {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Pack::GetStorage(get_storage) => Debug::fmt(get_storage, fmt),
            Pack::AlreadyTightPack(type_id) => fmt.write_fmt(format_args!(
                "The storage of type ({:?}) is already tightly packed.",
                type_id
            )),
            Pack::AlreadyLoosePack(type_id) => fmt.write_fmt(format_args!(
                "The storage of type ({:?}) is already loosely packed.",
                type_id
            )),
            Pack::AlreadyUpdatePack(type_id) => fmt.write_fmt(format_args!(
                "The storage of type ({:?}) is already has an update pack.",
                type_id
            )),
            Pack::UniqueStorage(name) => fmt.write_fmt(format_args!(
                "The storage of type \"{:?}\" is a unique storage and can't be packed.",
                name
            )),
        }
    }
}

impl Display for Pack {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        Debug::fmt(self, fmt)
    }
}

/// When removing components if one of them is packed owned, all storages packed
/// with it must be passed to the function.
///
/// This error occurs when there is a missing storage, `TypeId` will indicate which storage.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Remove {
    MissingPackStorage(TypeId),
}

impl Error for Remove {}

impl Debug for Remove {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Remove::MissingPackStorage(type_id) => fmt.write_fmt(format_args!("Missing storage for type ({:?}). To remove a packed component you have to pass all storages packed with it. Even if you just remove one component.", type_id))
        }
    }
}

impl Display for Remove {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        Debug::fmt(self, fmt)
    }
}

/// Trying to set the default workload to a non existant one will result in this error.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SetDefaultWorkload {
    Borrow(Borrow),
    MissingWorkload,
}

impl Error for SetDefaultWorkload {}

impl From<Borrow> for SetDefaultWorkload {
    fn from(borrow: Borrow) -> Self {
        SetDefaultWorkload::Borrow(borrow)
    }
}

impl Debug for SetDefaultWorkload {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        match self {
            SetDefaultWorkload::Borrow(borrow) => match borrow {
                Borrow::Unique => {
                    fmt.write_str("Cannot mutably borrow pipeline while it's already borrowed.")
                }
                Borrow::Shared => unreachable!(),
            },
            SetDefaultWorkload::MissingWorkload => {
                fmt.write_str("No workload with this name exists.")
            }
        }
    }
}

impl Display for SetDefaultWorkload {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        Debug::fmt(self, fmt)
    }
}

/// Try to run a non existant workload.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RunWorkload {
    Borrow(Borrow),
    MissingWorkload,
}

impl Error for RunWorkload {}

impl From<Borrow> for RunWorkload {
    fn from(borrow: Borrow) -> Self {
        RunWorkload::Borrow(borrow)
    }
}

impl Debug for RunWorkload {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        match self {
            RunWorkload::Borrow(borrow) => match borrow {
                Borrow::Unique => unreachable!(),
                Borrow::Shared => {
                    fmt.write_str("Cannot mutably borrow pipeline while it's already borrowed.")
                }
            },
            RunWorkload::MissingWorkload => fmt.write_str("No workload with this name exists."),
        }
    }
}

impl Display for RunWorkload {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        Debug::fmt(self, fmt)
    }
}

/// Error occuring when trying to sort a single packed storage.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Sort {
    MissingPackStorage,
    TooManyStorages,
}

impl Error for Sort {}

impl Debug for Sort {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Sort::MissingPackStorage => fmt.write_str("The storage you want to sort is packed, you may be able to sort the whole pack by passing all storages packed with it to the function. Some packs can't be sorted."),
            Sort::TooManyStorages => fmt.write_str("You provided too many storages non packed together. Only single storage and storages packed together can be sorted."),
        }
    }
}

impl Display for Sort {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        Debug::fmt(self, fmt)
    }
}
