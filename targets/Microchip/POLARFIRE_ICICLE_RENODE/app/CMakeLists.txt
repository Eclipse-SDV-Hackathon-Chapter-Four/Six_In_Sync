#
# Copyright (c) 2026 Eclipse ThreadX contributors
#
# This program and the accompanying materials are made available
# under the terms of the MIT license which is available at
# https://opensource.org/licenses/MIT.
#
# SPDX-License-Identifier: MIT
#

# Glob all common C source files for ThreadX with CONFIGURE_DEPENDS
file(GLOB THREADX_COMMON_SOURCES CONFIGURE_DEPENDS "${THREADX_DIR}/common/src/*.c")

# ThreadX Library Target for RISC-V 64-bit GNU Port
add_library(threadx STATIC
    ${THREADX_COMMON_SOURCES}
    ${CMAKE_CURRENT_SOURCE_DIR}/common/startup/tx_initialize_low_level.S
    ${THREADX_DIR}/ports/risc-v64/gnu/src/tx_thread_context_restore.S
    ${THREADX_DIR}/ports/risc-v64/gnu/src/tx_thread_context_save.S
    ${THREADX_DIR}/ports/risc-v64/gnu/src/tx_thread_interrupt_control.S
    ${THREADX_DIR}/ports/risc-v64/gnu/src/tx_thread_schedule.S
    ${THREADX_DIR}/ports/risc-v64/gnu/src/tx_thread_stack_build.S
    ${THREADX_DIR}/ports/risc-v64/gnu/src/tx_thread_system_return.S
    ${THREADX_DIR}/ports/risc-v64/gnu/src/tx_timer_interrupt.S
)

target_include_directories(threadx
    PUBLIC
        ${THREADX_DIR}/common/inc
        ${THREADX_DIR}/ports/risc-v64/gnu/inc
        ${CMAKE_CURRENT_SOURCE_DIR}/../lib/bsp/include
)

# -------------------------------------------------------------
# PolarFire SoC Icicle Kit Telemetry Executable
# -------------------------------------------------------------
add_executable(polarfire_icicle_demo
    main.c
    common/startup/entry.S
    common/startup/newlib_stubs.c
)

set_target_properties(polarfire_icicle_demo PROPERTIES SUFFIX ".elf")

target_include_directories(polarfire_icicle_demo
    PRIVATE
        ${CMAKE_CURRENT_SOURCE_DIR}/../lib/bsp/include
        ${SAMPLEX_ROOT_DIR}/bsp/include
)

option(ENABLE_FAULT_INJECTION "Enable deliberate synchronous fault injection for trap handler testing" OFF)
if(ENABLE_FAULT_INJECTION)
    target_compile_definitions(polarfire_icicle_demo PRIVATE TEST_FAULT_INJECTION=1)
endif()

target_link_libraries(polarfire_icicle_demo
    PRIVATE
        -Wl,--start-group
        polarfire_bsp
        threadx
        -Wl,--end-group
)
