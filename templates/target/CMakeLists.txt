#  Copyright (c) 2026 Eclipse ThreadX contributors
# 
#  This program and the accompanying materials are made available 
#  under the terms of the MIT license which is available at
#  https://opensource.org/licenses/MIT.
# 
#  SPDX-License-Identifier: MIT

cmake_minimum_required(VERSION 3.15)

# TODO: Replace 'TARGET_BOARD_TEMPLATE' with your target board name (e.g. MY_CUSTOM_BOARD)
project(TARGET_BOARD_TEMPLATE C CXX ASM)

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)

# Define root paths
get_filename_component(WORKSPACE_ROOT "${CMAKE_CURRENT_LIST_DIR}/../../.." ABSOLUTE)
set(SHARED_LIB_DIR "${WORKSPACE_ROOT}/libs")
set(SHARED_APP_DIR "${WORKSPACE_ROOT}/apps")
set(SHARED_BSP_DIR "${WORKSPACE_ROOT}/bsp")
set(SHARED_CMAKE_DIR "${WORKSPACE_ROOT}/cmake")

# Include GNU ARM Toolchain setup if building for bare-metal ARM Cortex-M
if(EXISTS "${SHARED_CMAKE_DIR}/gcc-arm-none-eabi.cmake")
    include("${SHARED_CMAKE_DIR}/gcc-arm-none-eabi.cmake")
endif()

# Global compiler flags
add_compile_options(
    -g3
    -ffunction-sections
    -fdata-sections
    -fno-strict-aliasing
    -fno-builtin
    -fno-common
    -Wall
    -Wshadow
    -Wdouble-promotion
    -Werror
    -Wno-unused-parameter
)

# Global preprocessor definitions
add_compile_definitions(
    TX_INCLUDE_USER_DEFINE_FILE
)

# TODO: Add vendor SDK / HAL subdirectories here if needed
# Example: add_subdirectory(lib/vendor_hal)

# Add libraries and application targets
add_subdirectory(lib)
add_subdirectory(app)
