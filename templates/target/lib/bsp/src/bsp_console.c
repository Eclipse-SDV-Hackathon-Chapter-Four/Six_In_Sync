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

/* TODO: Include vendor UART / Serial hardware headers here */

void bsp_console_init(void)
{
#if BSP_HAS_CONSOLE
    /* TODO: Enable UART peripheral and GPIO clocks.
     * Configure RX/TX pins for alternate function serial mode and configure baud rate to BSP_UART_BAUDRATE.
     */
#endif
}

void bsp_console_write(const char *data, size_t length)
{
#if BSP_HAS_CONSOLE
    /* TODO: Transmit character array over serial UART hardware */
    (void)data;
    (void)length;
#else
    (void)data;
    (void)length;
#endif
}
