# Target Linker Scripts

Place your target MCU linker script (`.ld` for GCC, `.icf` for IAR, or `.sct` for Keil ARMClang) here.

## Recommended Files:
- `<mcu_part_number>.ld` (Memory region definitions for FLASH, SRAM, CCMRAM, and section mapping)

## CMake Integration:
Specify the linker script path in `targets/<Vendor>/<BOARD_NAME>/app/CMakeLists.txt`:
```cmake
set_target_linker(${PROJECT_NAME} ${CMAKE_CURRENT_SOURCE_DIR}/common/linker/<mcu_part_number>.ld)
```
