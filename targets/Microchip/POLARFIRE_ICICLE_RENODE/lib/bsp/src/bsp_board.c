/*
 * Copyright (c) 2026 Eclipse ThreadX contributors
 *
 * This program and the accompanying materials are made available
 * under the terms of the MIT license which is available at
 * https://opensource.org/licenses/MIT.
 *
 * SPDX-License-Identifier: MIT
 */

#include <stdint.h>
#include "bsp/board.h"
#include "bsp/console.h"
#include "bsp/led.h"
#include "board_config.h"

extern void board_init(void);

void bsp_board_init(void) {
    /* Initialize PolarFire hardware peripherals */
    board_init();

    bsp_console_init();
    bsp_led_init();
}

uint32_t bsp_board_get_system_clock(void) {
    return (uint32_t)BSP_SYSTEM_CLOCK_HZ;
}
