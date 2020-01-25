use std::hash::Hash;

pub trait Entity {
    type Id: Eq + Clone + Hash;
    fn id(&self) -> &Self::Id;
}
