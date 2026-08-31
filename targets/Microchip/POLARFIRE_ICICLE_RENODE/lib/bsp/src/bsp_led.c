/*
 * Copyright (c) 2026 Eclipse ThreadX contributors
 *
 * This program and the accompanying materials are made available
 * under the terms of the MIT license which is available at
 * https://opensource.org/licenses/MIT.
 *
 * SPDX-License-Identifier: MIT
 */

#include <stdbool.h>
#include "bsp/led.h"
#include "board_config.h"

static bool s_virtual_led_state = false;

void bsp_led_init(void) {
#if BSP_HAS_LED
    s_virtual_led_state = false;
#endif
}

void bsp_led_on(void) {
#if BSP_HAS_LED
    s_virtual_led_state = true;
#endif
}

void bsp_led_off(void) {
#if BSP_HAS_LED
    s_virtual_led_state = false;
#endif
}

void bsp_led_toggle(void) {
#if BSP_HAS_LED
    s_virtual_led_state = !s_virtual_led_state;
#endif
}
