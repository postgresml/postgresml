Using dmlc-core with CMake
==========================
dmlc defines a exported CMake target which can be used by `find_package` command.

For example, if you have a simple C++ project that contains only a main.cc file,
which uses dmlc-core as dependency, the CMakeLists.txt for your project can be
defined as follow:

``` cmake
project(demo)
cmake_minimum_required(VERSION 3.2)

find_package(dmlc REQUIRED)
add_executable(demo main.cc)
target_link_libraries(demo dmlc::dmlc)
```