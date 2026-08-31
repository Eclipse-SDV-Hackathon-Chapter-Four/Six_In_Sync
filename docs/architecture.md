# Eclipse ThreadX BSP Framework Architecture

This document describes the architecture, design philosophy, directory structure, and onboarding process for the reusable Board Support Package (BSP) framework.

---

## 1. Design Philosophy

The BSP framework is designed to be **additive and non-invasive**, allowing new boards to be integrated without modifying existing board implementations.

1. **Legacy Isolation**: The board directories that predate this framework (`/MXChip`, `/OpenHW`, `/STMicroelectronics`) remain completely untouched, preserving their drivers, submodules, and build systems.
2. **Platform-Independent Applications**: Target applications use only the abstract BSP interfaces and have no compile-time dependency on vendor-specific HALs, SDKs, or hardware registers.
3. **Reusable Infrastructure**: Shared CMake toolchains and build utilities are centralized under `/cmake` to eliminate duplicated build configuration across supported boards.

---

## 2. Directory Structure

```text
samplex/ (repository root)
├── libs/                           # Shared RTOS components (ThreadX, NetX Duo, FileX, USBX)
├── scripts/                        # Repository-wide helper scripts
├── MXChip/                         # [Pre-framework] Standalone board sample
├── OpenHW/                         # [Pre-framework] Standalone board sample
├── STMicroelectronics/             # [Pre-framework] Standalone board samples
├── targets/                        # [Framework] Supported BSP target boards
│   └── Microchip/
│       └── POLARFIRE_ICICLE_RENODE/ # Board-specific BSP implementation & Renode target
├── bsp/                            # [Framework] Abstract BSP interface definitions
│   └── include/bsp/                # board.h, led.h, console.h
├── cmake/                          # [Framework] Shared CMake configuration and utilities
├── docs/                           # [Framework] Architecture and onboarding documentation
└── templates/                      # [Framework] Templates for onboarding new boards
```

---

## 3. BSP Interface Contract

Every board added to the framework under `/targets` must implement the abstract APIs defined in `/bsp/include/bsp/`.

### Core Board Control (`board.h`)

* `void bsp_board_init(void)`: Initializes the board, including system clocks, GPIO, and required peripherals.

### LED Control (`led.h`)

* `void bsp_led_init(void)`: Configures the board's user LED.
* `void bsp_led_on(void)`: Turns the LED on.
* `void bsp_led_off(void)`: Turns the LED off.
* `void bsp_led_toggle(void)`: Toggles the LED state.

### Serial Console (`console.h`)

* `void bsp_console_init(void)`: Initializes the default UART console.
* `void bsp_console_write(const char *data, size_t length)`: Transmits a block of data over the console interface.

---

## 4. How to Onboard a New Board

1. **Create the Target Folder**: Create a new directory under `targets/<Vendor>/<Board_Name>/` using `/templates/target/` as the starting point.
2. **Define Local Configuration**: Create a `board_config.h` file containing board-specific settings such as clock configuration, UART parameters, and ThreadX memory allocation.
3. **Implement the BSP APIs**: Implement the interfaces defined in `/bsp/include/bsp/` using the vendor SDK or direct register access.
4. **Configure CMake**: Add the board target to `CMakeLists.txt`, build the BSP as a static library, and link it with the desired application from `/apps/`.

Once a board implements the required BSP interfaces, any compatible application under `/apps` can be built for that board without modifying the application source.
