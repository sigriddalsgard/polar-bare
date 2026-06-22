cmake_minimum_required(VERSION 4.0)

find_package(cmake-bare REQUIRED PATHS node_modules/cmake-bare)
find_package(cmake-cargo REQUIRED PATHS node_modules/cmake-cargo)

project(polar_bare_addon C)

add_crate()

add_bare_module(polar_bare_addon)

target_sources(
  ${polar_bare_addon}
  PRIVATE
    binding.c
)

target_link_libraries(
  ${polar_bare_addon}
  PRIVATE
    polar_bare
)
