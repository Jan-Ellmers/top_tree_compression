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

void set_structure_vector_c(int length, const bool* new_structure);

int get_structure_vector_length_c();

void get_structure_vector_c(bool* new_structure);

//-----------------------------------------------------------------------------------\\

void set_pointer_vector_c(int length, const size_t* new_pointer);

int get_pointer_vector_length_c();

void get_pointer_vector_c(size_t* new_pointer);

//-----------------------------------------------------------------------------------\\

void set_merge_type_vector_c(int length, const int* new_merge_type);

int get_merge_type_vector_length_c();

void get_merge_type_vector_c(int* new_merge_type);

//-----------------------------------------------------------------------------------\\

void set_label_vector_c(const char* new_label);

int get_label_vector_length_c();

void get_label_vector_c(char* new_label);

//-----------------------------------------------------------------------------------\\

#ifdef __cplusplus
}
#endif
