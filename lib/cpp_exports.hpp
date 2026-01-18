#pragma once
#include "target\cxxbridge\rarc_lib\src\cpp_exports.rs.h"
#include "target\cxxbridge\rarc_lib\src\cpp_exports.rs.cc"

namespace rarc_lib 
{

enum class Endian : uint8_t {
    Big,
    Little
};

constexpr uint8_t to_u8(Endian endian) {
    return static_cast<uint8_t>(endian);
}

enum class FileAttr : uint8_t {
    FILE = 0x1,
    FOLDER = 0x2,
    COMPRESSED = 0x4,
    MRAM = 0x10,
    ARAM = 0x20,
    DVD = 0x40,
    SZS = 0x80
};

constexpr uint8_t to_u8(FileAttr attr) {
    return static_cast<uint8_t>(attr);
}

constexpr FileAttr operator|(FileAttr left, FileAttr right) {
    return static_cast<FileAttr>(to_u8(left) | to_u8(right));
}

constexpr FileAttr DEFAULT = FileAttr::FILE | FileAttr::MRAM;

std::string archive_to_dir(const std::vector<uint8_t>& data, const std::string& output) {
    auto result = librarc::archive_to_dir(data, output);
    return result.c_str();
}

std::vector<uint8_t> dir_to_archive(const std::string& path, FileAttr attr, Endian endian) {
    auto result = librarc::dir_to_archive(path, to_u8(attr), to_u8(endian));
    return *result;
}
}