#ifdef __cplusplus
extern "C" {
#endif
#include <cstddef>

//-----------------------------------------------------------------------------------\\

bool load_structure_from_file_c(const char* path);

bool load_pointer_from_file_c(const char* path);

bool load_merge_type_from_file_c(const char* path);

bool load_label_from_file_c(const char* path);

//-----------------------------------------------------------------------------------\\

bool save_structure_to_file_c(const char* path);

bool save_pointer_to_file_c(const char* path);

bool save_merge_type_to_file_c(const char* path);

bool save_label_to_file_c(const char* path);

//-----------------------------------------------------------------------------------\\

void set_structure_vector_c(size_t length, const bool* new_structure);

size_t get_structure_vector_length_c();

void get_structure_vector_c(bool* new_structure);

//-----------------------------------------------------------------------------------\\

void set_pointer_vector_c(size_t length, const size_t* new_pointer);

size_t get_pointer_vector_length_c();

void get_pointer_vector_c(size_t* new_pointer);

//-----------------------------------------------------------------------------------\\

void set_merge_type_vector_c(size_t length, const int* new_merge_type);

size_t get_merge_type_vector_length_c();

void get_merge_type_vector_c(int* new_merge_type);

//-----------------------------------------------------------------------------------\\

void set_label_vector_c(const char* new_label);

size_t get_label_vector_length_c();

void get_label_vector_c(char* new_label);

//-----------------------------------------------------------------------------------\\

#ifdef __cplusplus
}
#endif
