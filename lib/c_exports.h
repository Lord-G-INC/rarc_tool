#pragma once
#ifdef __cplusplus
extern "C" {
#endif

enum FileAttr : unsigned char {
    FILE = 0x1,
    FOLDER = 0x2,
    COMPRESSED = 0x4,
    MRAM = 0x10,
    ARAM = 0x20,
    DVD = 0x40,
    SZS = 0x80
};

bool archive_to_dir(const unsigned char* buffer, unsigned long size, const char* dir);

bool dir_to_archive(const char* buffer, FileAttr attr, const char* file);

#ifdef __cplusplus
}
#endif