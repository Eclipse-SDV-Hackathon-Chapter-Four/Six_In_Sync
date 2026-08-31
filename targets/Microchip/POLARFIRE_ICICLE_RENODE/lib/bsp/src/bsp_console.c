/*
 * Copyright (c) 2026 Eclipse ThreadX contributors
 *
 * This program and the accompanying materials are made available
 * under the terms of the MIT license which is available at
 * https://opensource.org/licenses/MIT.
 *
 * SPDX-License-Identifier: MIT
 */

#include "bsp/console.h"
#include "board_config.h"
#include <stddef.h>

extern void uart_init(void);
extern void uart_write(const char *data, size_t len);

void bsp_console_init(void) {
#if BSP_HAS_CONSOLE
    uart_init();
#endif
}

void bsp_console_write(const char *data, size_t length) {
#if BSP_HAS_CONSOLE
    if (!data || length == 0) return;
    uart_write(data, length);
#endif
}
