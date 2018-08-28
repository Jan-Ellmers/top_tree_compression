#![allow(dead_code)]
pub extern crate libc;
use self::libc::{c_int, wchar_t, size_t};
use std::cell::RefCell;


#[link(name = "sdsl_interface", kind="static")]
extern "C" {

//-----------------------------------------------------------------------------------\\

    fn load_structure_from_file_c(path: *const wchar_t) -> bool;

    fn load_pointer_from_file_c(path: *const wchar_t) -> bool;

    fn load_merge_type_from_file_c(path: *const wchar_t) -> bool;

    fn load_label_from_file_c(path: *const wchar_t) -> bool;

//-----------------------------------------------------------------------------------\\

    fn save_structure_to_file_c(path: *const wchar_t) -> bool;

    fn save_pointer_to_file_c(path: *const wchar_t) -> bool;

    fn save_merge_type_to_file_c(path: *const wchar_t) -> bool;

    fn save_label_to_file_c(path: *const wchar_t) -> bool;

//-----------------------------------------------------------------------------------\\

    fn set_structure_vector_c(length: c_int, new_structure: *const bool);

    fn get_structure_vector_length_c() -> c_int;

    fn get_structure_vector_c(new_structure: *mut bool);

//-----------------------------------------------------------------------------------\\

    fn set_pointer_vector_c(length: c_int, new_pointer: *const size_t);

    fn get_pointer_vector_length_c() -> c_int;

    fn get_pointer_vector_c(new_pointer: *mut size_t);

//-----------------------------------------------------------------------------------\\

    fn set_merge_type_vector_c(length: c_int, new_merge_type: *const c_int);

    fn get_merge_type_vector_length_c() -> c_int;

    fn get_merge_type_vector_c(new_merge_type: *mut c_int);

//-----------------------------------------------------------------------------------\\

    fn set_label_vector_c(new_label: *const wchar_t);

    fn get_label_vector_length_c() -> c_int;

    fn get_label_vector_c(new_label: *mut wchar_t);

//-----------------------------------------------------------------------------------\\
}

thread_local! {
    static STRUCTURE_INITIALSIED: RefCell<bool> = RefCell::new(false);
    static POINTER_INITIALSIED: RefCell<bool> = RefCell::new(false);
    static MERGE_TYPE_INITIALSIED: RefCell<bool> = RefCell::new(false);
    static LABEL_INITIALSIED: RefCell<bool> = RefCell::new(false);
}

//-----------------------------------------------------------------------------------\\

pub fn load_structure_from_file(path: &str) -> bool {
    let success;
    let mut path = path.to_owned();
    path.push(char::from(0 as u8));
    unsafe {
        success = load_structure_from_file_c(path.as_ptr() as *const i32);
    }
    STRUCTURE_INITIALSIED.with(|elem| *elem.borrow_mut() = success);
    success
}

pub fn load_pointer_from_file(path: &str) -> bool {
    let success;
    let mut path = path.to_owned();
    path.push(char::from(0 as u8));
    unsafe {
        success = load_pointer_from_file_c(path.as_ptr() as *const i32);
    }
    POINTER_INITIALSIED.with(|elem| *elem.borrow_mut() = success);
    success
}

pub fn load_merge_type_from_file(path: &str) -> bool {
    let success;
    let mut path = path.to_owned();
    path.push(char::from(0 as u8));
    unsafe {
        success = load_merge_type_from_file_c(path.as_ptr() as *const i32);
    }
    MERGE_TYPE_INITIALSIED.with(|elem| *elem.borrow_mut() = success);
    success
}

pub fn load_label_from_file(path: &str) -> bool {
    let success;
    let mut path = path.to_owned();
    path.push(char::from(0 as u8));
    unsafe {
        success = load_label_from_file_c(path.as_ptr() as *const i32);
    }
    LABEL_INITIALSIED.with(|elem| *elem.borrow_mut() = success);
    success
}

//-----------------------------------------------------------------------------------\\

pub fn save_structure_to_file(path: &str) -> bool {
    assert!(STRUCTURE_INITIALSIED.with(|elem| *elem.borrow()));
    let mut path = path.to_owned();
    path.push(char::from(0 as u8));
    unsafe {
        save_structure_to_file_c(path.as_ptr() as *const i32)
    }
}

pub fn save_pointer_to_file(path: &str) -> bool {
    assert!(POINTER_INITIALSIED.with(|elem| *elem.borrow()));
    let mut path = path.to_owned();
    path.push(char::from(0 as u8));
    unsafe {
        save_pointer_to_file_c(path.as_ptr() as *const i32)
    }
}

pub fn save_merge_type_to_file(path: &str) -> bool {
    assert!(MERGE_TYPE_INITIALSIED.with(|elem| *elem.borrow()));
    let mut path = path.to_owned();
    path.push(char::from(0 as u8));
    unsafe {
        save_merge_type_to_file_c(path.as_ptr() as *const i32)
    }
}

pub fn save_label_to_file(path: &str) -> bool {
    assert!(LABEL_INITIALSIED.with(|elem| *elem.borrow()));
    let mut path = path.to_owned();
    path.push(char::from(0 as u8));
    unsafe {
        save_label_to_file_c(path.as_ptr() as *const i32)
    }
}

//-----------------------------------------------------------------------------------\\

pub fn set_structure_vector(new_structure: Vec<bool>) {
    STRUCTURE_INITIALSIED.with(|elem| *elem.borrow_mut() = true);
    unsafe {
        set_structure_vector_c(new_structure.len() as i32, new_structure.as_ptr());
    }
}

pub fn get_structure_vector() -> Vec<bool> {
    assert!(STRUCTURE_INITIALSIED.with(|elem| *elem.borrow()));
    let length;
    unsafe {
        length = get_structure_vector_length_c();
    }
    let mut structure = vec!(false; length as usize);

    unsafe {
        get_structure_vector_c(structure.as_mut_ptr());
    }

    structure
}

//-----------------------------------------------------------------------------------\\

pub fn set_pointer_vector(new_pointer: Vec<usize>) {
    POINTER_INITIALSIED.with(|elem| *elem.borrow_mut() = true);
    unsafe {
        set_pointer_vector_c(new_pointer.len() as i32, new_pointer.as_ptr() as *const usize);
    }
}

pub fn get_pointer_vector() -> Vec<usize> {
    assert!(POINTER_INITIALSIED.with(|elem| *elem.borrow()));
    let length;
    unsafe {
        length = get_pointer_vector_length_c();
    }
    let mut pointer: Vec<usize> = vec!(0; length as usize);

    unsafe {
        get_pointer_vector_c(pointer.as_mut_ptr());
    }

    pointer
}

//-----------------------------------------------------------------------------------\\

pub fn set_merge_type_vector(new_merge_type: Vec<i32>) {
    MERGE_TYPE_INITIALSIED.with(|elem| *elem.borrow_mut() = true);
    unsafe {
        set_merge_type_vector_c(new_merge_type.len() as i32, new_merge_type.as_ptr());
    }
}

pub fn get_merge_type_vector() -> Vec<i32> {
    assert!(MERGE_TYPE_INITIALSIED.with(|elem| *elem.borrow()));
    let length;
    unsafe {
        length = get_merge_type_vector_length_c();
    }
    let mut merge_type = vec!(0; length as usize);

    unsafe {
        get_merge_type_vector_c(merge_type.as_mut_ptr());
    }

    merge_type
}

//-----------------------------------------------------------------------------------\\

pub fn set_label_vector_from_string(new_label: Vec<String>) {
    let mut string = String::new();
    for elem in new_label {
        assert!(elem.find('\n').is_none());
        string.push_str(&elem);
        string.push('\n');
    }
    LABEL_INITIALSIED.with(|elem| *elem.borrow_mut() = true);
    let string_pointer = string.as_ptr() as *const i32;

    unsafe {
        set_label_vector_c(string_pointer);
    }
}

pub fn set_label_vector_from_str(new_label: Vec<&str>) {
    let mut string = String::new();
    for elem in new_label {
        assert!(elem.find('\n').is_none());
        string.push_str(&elem);
        string.push('\n');
    }
    LABEL_INITIALSIED.with(|elem| *elem.borrow_mut() = true);
    let string_pointer = string.as_ptr() as *const i32;

    unsafe {
        set_label_vector_c(string_pointer);
    }
}

pub fn get_label_vector() -> Vec<String> {
    assert!(LABEL_INITIALSIED.with(|elem| *elem.borrow()));
    let length;
    unsafe {
        length = get_label_vector_length_c();
    }
    let mut string = String::new();
    for _index in 0..length {
        string.push('\n');
    }
    unsafe {
        get_label_vector_c(string.as_ptr() as *mut i32);
    }

    let mut to_return: Vec<String> = string.split('\n').map(|elem| elem.to_owned()).collect();
    to_return.pop();
    to_return

}

//-----------------------------------------------------------------------------------\\