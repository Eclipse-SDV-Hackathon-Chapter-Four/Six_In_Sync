/*
 * Copyright (c) 2026 Eclipse ThreadX contributors
 *
 * This program and the accompanying materials are made available
 * under the terms of the MIT license which is available at
 * https://opensource.org/licenses/MIT.
 *
 * SPDX-License-Identifier: MIT
 */

#ifndef POLARFIRE_UART_H
#define POLARFIRE_UART_H

#include <stddef.h>
#include <stdint.h>

#define MMUART1_BASE    0x20100000ULL

void uart_init(void);
void uart_putc(char ch);
void uart_puts(const char *str);
void uart_write(const char *data, size_t len);
int uart_has_rx(void);
char uart_getc(void);

#endif /* POLARFIRE_UART_H */
