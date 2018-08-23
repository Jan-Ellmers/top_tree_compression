use std::ops::{Deref, DerefMut};



#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Uninitialized<T> {
    pub value: Option<T>,
}

impl<T> Uninitialized<T> {
    pub fn new() -> Uninitialized<T> {
        Uninitialized {
            value: None,
        }
    }

    pub fn into_inner(mut self) -> T {
        self.value.take().unwrap()
    }

    pub fn try_into_inner(mut self) -> Option<T> {
        self.value.take()
    }

    pub fn set_value(&mut self, new_value: T) {
        self.value = Some(new_value);
    }

    /*pub fn swap_value(&mut self, new_value: T) -> T {
        let to_return;
        if let Some(data) = self.value.take() {
            to_return = data;
        } else {
            panic!("Error: Called swap_value on a SaveOption before feeding it any value");
        }
        self.value = Some(new_value);
        to_return
    }*/

    pub fn is_initialized(&self) -> bool {
        self.value.is_some()
    }

    pub fn is_uninitialized(&self) -> bool {
        self.value.is_none()
    }
}

impl<T> Deref for Uninitialized<T> {
    type Target = T;

    fn deref(&self) -> &T {
        if let Some(ref data) = self.value {
            data
        } else {
            panic!("Error: Use of a not initialized value");
        }
    }
}

impl<T> DerefMut for Uninitialized<T> {
    fn deref_mut(&mut self) -> &mut T {
        if let Some(ref mut data) = self.value {
            data
        } else {
            panic!("Error: Use of a not initialized value");
        }
    }
}