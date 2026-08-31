# Target MCU Startup Files

Place your target MCU startup assembly file and CMSIS system initialization file here.

## Recommended Files:
- `startup_<mcu>.s` or `startup_<mcu>.c` (Vector table definition & Reset_Handler)
- `system_<mcu>.c` (SystemCoreClock update & CMSIS system initialization)
- `tx_initialize_low_level.S` (ThreadX low-level architecture initialization for your Cortex-M core)

## CMake Integration:
Reference these files in `targets/<Vendor>/<BOARD_NAME>/app/CMakeLists.txt` under the `SOURCES` list.
