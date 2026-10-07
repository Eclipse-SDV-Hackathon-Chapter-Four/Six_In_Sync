cmake_minimum_required(VERSION 3.20)
project(threadx_temp_sensor CXX C ASM)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# ── Build-path selection ─────────────────────────────────────────────────────
# Linux / ThreadX-Linux-port  →  default  (Docker compose service)
# ARM  / Renode               →  cmake -DTHREADX_ARM_TARGET=ON  (Renode path)
option(THREADX_ARM_TARGET "Cross-compile for ARM Cortex-M4 / Renode" OFF)

# ── FetchContent setup ───────────────────────────────────────────────────────
include(FetchContent)
set(FETCHCONTENT_QUIET OFF)

# ── ThreadX arch for the Linux port ─────────────────────────────────────────
# Must be set before opensomeip is configured so that when
# SOMEIP_THREADX_LINUX_TESTS pulls in Eclipse ThreadX it uses the right port.
if(NOT THREADX_ARM_TARGET)
    set(THREADX_ARCH      "linux" CACHE STRING "" FORCE)
    set(THREADX_TOOLCHAIN "gnu"   CACHE STRING "" FORCE)
else()
    set(THREADX_ARCH      "cortex_m4" CACHE STRING "" FORCE)
    set(THREADX_TOOLCHAIN "gnu"       CACHE STRING "" FORCE)
endif()

# ── OpenSOME/IP (also fetches Eclipse ThreadX internally) ───────────────────
# SOMEIP_THREADX_LINUX_TESTS=ON tells opensomeip to fetch Eclipse ThreadX
# using its own tested tag and build the ThreadX Linux port.
# BUILD_TESTS=OFF suppresses the opensomeip test binaries; ThreadX is still
# fetched and made available as the azrtos::threadx CMake target.
FetchContent_Declare(
    opensomeip
    GIT_REPOSITORY https://github.com/vtz/opensomeip.git
    GIT_TAG        main
    GIT_SHALLOW    TRUE
)

set(SOMEIP_THREADX_LINUX_TESTS ON  CACHE BOOL "" FORCE)
set(SOMEIP_USE_LWIP            OFF CACHE BOOL "" FORCE)
set(BUILD_TESTS                OFF CACHE BOOL "" FORCE)
set(BUILD_EXAMPLES             OFF CACHE BOOL "" FORCE)
set(SOMEIP_DEV_TOOLS           OFF CACHE BOOL "" FORCE)

FetchContent_MakeAvailable(opensomeip)

# ── Temperature sensor executable ────────────────────────────────────────────
add_executable(threadx_temp_sensor
    src/main.cpp
    src/temperature_sensor.cpp
)

target_include_directories(threadx_temp_sensor PRIVATE
    ${CMAKE_SOURCE_DIR}/src
    ${opensomeip_SOURCE_DIR}/include
)

target_link_libraries(threadx_temp_sensor PRIVATE
    azrtos::threadx
    opensomeip          # provides SOME/IP platform primitives
)

if(THREADX_ARM_TARGET)
    target_compile_definitions(threadx_temp_sensor PRIVATE THREADX_ARM_TARGET=1)
    target_link_libraries(threadx_temp_sensor PRIVATE
        # lwIP must be provided by the BSP / integration layer when targeting ARM
        # lwipcore lwiparch   ← add when integrating BSP
    )
else()
    # Linux: link pthreads (required by ThreadX Linux port)
    find_package(Threads REQUIRED)
    target_link_libraries(threadx_temp_sensor PRIVATE Threads::Threads)
endif()

# ── Installation ─────────────────────────────────────────────────────────────
install(TARGETS threadx_temp_sensor DESTINATION bin)
