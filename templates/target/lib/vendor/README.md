# Vendor Hardware Abstraction Libraries (HAL / SDK)

Place vendor-supplied hardware abstraction drivers or SDK source code here if your target board does not use a system-installed CMake package manager.

## Integration Patterns:

### Option A: Embedded Source Tree (e.g., STM32Cube HAL)
Place the driver source files here and register a static library in `targets/<Vendor>/<BOARD_NAME>/lib/CMakeLists.txt`:
```cmake
add_subdirectory(vendor/stm32cubef4)
```

### Option B: External SDK Package (e.g., Raspberry Pi Pico SDK)
If using an external CMake package manager, do not place files here. Instead, call `find_package(...)` or include the SDK's CMake entry point in `targets/<Vendor>/<BOARD_NAME>/CMakeLists.txt`.
