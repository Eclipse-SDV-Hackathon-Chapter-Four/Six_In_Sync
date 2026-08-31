/*
 * Copyright (c) 2026 Eclipse ThreadX contributors
 *
 * This program and the accompanying materials are made available
 * under the terms of the MIT license which is available at
 * https://opensource.org/licenses/MIT.
 *
 * SPDX-License-Identifier: MIT
 */

#ifndef BSP_CONSOLE_H
#define BSP_CONSOLE_H

#include <stddef.h>

/**
 * @brief Initialize the serial console peripheral and pin muxing.
 */
void bsp_console_init(void);

/**
 * @brief Write a buffer of data to the serial console.
 * 
 * @param data Pointer to the character buffer to send.
 * @param length Number of characters to transmit.
 */
void bsp_console_write(const char *data, size_t length);

#endif /* BSP_CONSOLE_H */
