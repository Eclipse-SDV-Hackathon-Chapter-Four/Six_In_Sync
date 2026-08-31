/*
 * Copyright (c) 2026 Eclipse ThreadX contributors
 *
 * This program and the accompanying materials are made available
 * under the terms of the MIT license which is available at
 * https://opensource.org/licenses/MIT.
 *
 * SPDX-License-Identifier: MIT
 */

#include "bsp/board.h"
#include "board_config.h"

/* TODO: Include vendor hardware HAL or register definitions here (e.g. #include "mcu_hal.h") */

/**
 * @brief Initialize the system core (clocks, flash latency, system configuration).
 */
void bsp_board_init(void)
{
    /* TODO: 1. Initialize Low-level Hardware Abstraction Layer / Power Regulators if applicable */

    /* TODO: 2. Configure System Clock Tree (Oscillators, PLL, Bus Dividers) to run at BSP_SYSTEM_CLOCK_HZ */

    /* TODO: 3. Configure Flash Read Latency / Wait states matching the CPU clock frequency */

    /* TODO: 4. Configure OS Tick Timer if MCU HAL requires a separate hardware timer to keep SysTick free for ThreadX */
}
