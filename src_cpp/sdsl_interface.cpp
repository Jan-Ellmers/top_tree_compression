#include "sdsl_interface.h"
#include <sdsl/bit_vectors.hpp>
#include <sdsl/wt_huff.hpp>
#include <sdsl/wavelet_trees.hpp>
#include <sdsl/bit_vectors.hpp>
#include <cstddef>

using namespace sdsl;

rrr_vector<> structure;
vlc_vector<> pointer;
vlc_vector<> merge_type;
wt_huff<> label;

//-----------------------------------------------------------------------------------\\

bool load_structure_from_file_c(const char* path) {
    return load_from_file(structure, path);
}

bool load_pointer_from_file_c(const char* path) {
    return load_from_file(pointer, path);
}

bool load_merge_type_from_file_c(const char* path) {
    return load_from_file(merge_type, path);
}

bool load_label_from_file_c(const char* path) {
    return load_from_file(label, path);
}

//-----------------------------------------------------------------------------------\\

bool save_structure_to_file_c(const char* path) {
    return store_to_file(structure, path);
}

bool save_pointer_to_file_c(const char* path) {
    return store_to_file(pointer, path);
}

bool save_merge_type_to_file_c(const char* path) {
    return store_to_file(merge_type, path);
}

bool save_label_to_file_c(const char* path) {
    return store_to_file(label, path);
}

//-----------------------------------------------------------------------------------\\

void set_structure_vector_c(int length, const bool* new_structure) {
    bit_vector b(length, 0);
    for (unsigned int volatile i = 0; i < length; i++) {
        b[i] = new_structure[i];
    }
    structure = rrr_vector<>(b);
}

int get_structure_vector_length_c() {
    return structure.size();
}

void get_structure_vector_c(bool* new_structure) {
    int length = get_structure_vector_length_c();
    for (unsigned int volatile i = 0; i < length; i++) {
        new_structure[i] = structure[i];
    }
}

//-----------------------------------------------------------------------------------\\

void set_pointer_vector_c(int length, const size_t* new_pointer) {
    int_vector<> v(length, 0);
    for (unsigned int volatile i = 0; i < length; i++) {
        v[i] = new_pointer[i];
    }
    pointer = vlc_vector<>(v);
}

int get_pointer_vector_length_c() {
    return pointer.size();
}

void get_pointer_vector_c(size_t* new_pointer) {
    int length = get_pointer_vector_length_c();
    for (unsigned int volatile i = 0; i < length; i++) {
        new_pointer[i] = pointer[i];
    }
}

//-----------------------------------------------------------------------------------\\

void set_merge_type_vector_c(int length, const int* new_merge_type) {
    int_vector<> v(length, 0);
    for (unsigned int volatile i = 0; i < length; i++) {
        v[i] = new_merge_type[i];
    }
    merge_type = vlc_vector<>(v);
}

int get_merge_type_vector_length_c() {
    return merge_type.size();
}

void get_merge_type_vector_c(int* new_merge_type) {
    int length = get_merge_type_vector_length_c();
    for (unsigned int volatile i = 0; i < length; i++) {
        new_merge_type[i] = merge_type[i];
    }
}

//-----------------------------------------------------------------------------------\\

void set_label_vector_c(const char* new_label) {
    construct_im(label, new_label, 1);
}

int get_label_vector_length_c() {
    return label.size();
}

void get_label_vector_c(char* new_label) {
    for (size_t i=0; i < get_label_vector_length_c(); ++i) {
        new_label[i] = label[i];
    }
}

//-----------------------------------------------------------------------------------\\