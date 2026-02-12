use crate::*;

pub trait Entity: Clone {
    fn _id(&self) -> Id;
}
