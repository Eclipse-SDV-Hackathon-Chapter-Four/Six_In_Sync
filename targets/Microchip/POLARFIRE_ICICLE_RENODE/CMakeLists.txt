#
# Copyright (c) 2026 Eclipse ThreadX contributors
#
# This program and the accompanying materials are made available
# under the terms of the MIT license which is available at
# https://opensource.org/licenses/MIT.
#
# SPDX-License-Identifier: MIT
#

cmake_minimum_required(VERSION 3.20)

set(CMAKE_SYSTEM_NAME Generic)
set(CMAKE_SYSTEM_PROCESSOR riscv64)

# Set 64-bit RISC-V Cross Compiler
if(DEFINED ENV{RISCV64_TOOLCHAIN_PATH})
    set(TOOLCHAIN_BIN_DIR "$ENV{RISCV64_TOOLCHAIN_PATH}/bin")
    find_program(CMAKE_C_COMPILER NAMES riscv-none-elf-gcc riscv64-none-elf-gcc riscv64-unknown-elf-gcc PATHS ${TOOLCHAIN_BIN_DIR} NO_DEFAULT_PATH REQUIRED)
    find_program(CMAKE_CXX_COMPILER NAMES riscv-none-elf-g++ riscv64-none-elf-g++ riscv64-unknown-elf-g++ PATHS ${TOOLCHAIN_BIN_DIR} NO_DEFAULT_PATH REQUIRED)
    find_program(CMAKE_ASM_COMPILER NAMES riscv-none-elf-gcc riscv64-none-elf-gcc riscv64-unknown-elf-gcc PATHS ${TOOLCHAIN_BIN_DIR} NO_DEFAULT_PATH REQUIRED)
else()
    find_program(CMAKE_C_COMPILER NAMES riscv-none-elf-gcc riscv64-none-elf-gcc riscv64-unknown-elf-gcc REQUIRED)
    find_program(CMAKE_CXX_COMPILER NAMES riscv-none-elf-g++ riscv64-none-elf-g++ riscv64-unknown-elf-g++ REQUIRED)
    find_program(CMAKE_ASM_COMPILER NAMES riscv-none-elf-gcc riscv64-none-elf-gcc riscv64-unknown-elf-gcc REQUIRED)
endif()

project(polarfire_icicle_renode C ASM)

# Root Directories
set(SAMPLEX_ROOT_DIR "${CMAKE_CURRENT_LIST_DIR}/../../..")
set(THREADX_DIR "${SAMPLEX_ROOT_DIR}/libs/threadx")

# Architecture flags for SiFive U54 64-Bit RISC-V Core (RV64GC / lp64d)
set(RISCV_FLAGS "-march=rv64gc -mabi=lp64d -mcmodel=medany")
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} ${RISCV_FLAGS} -O2 -g -Wall -Wextra")
set(CMAKE_ASM_FLAGS "${CMAKE_ASM_FLAGS} ${RISCV_FLAGS}")
set(CMAKE_EXE_LINKER_FLAGS "${RISCV_FLAGS} -T${CMAKE_CURRENT_LIST_DIR}/app/common/linker/linker.ld -nostartfiles -Wl,--gc-sections")

add_subdirectory(lib)
add_subdirectory(app)
