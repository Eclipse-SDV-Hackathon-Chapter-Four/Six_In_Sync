/*
 * Copyright (c) 2026 Eclipse ThreadX contributors
 *
 * This program and the accompanying materials are made available
 * under the terms of the MIT license which is available at
 * https://opensource.org/licenses/MIT.
 *
 * SPDX-License-Identifier: MIT
 */

#include "uart.h"

#define REG_THR     (*(volatile uint32_t *)(MMUART1_BASE + 0x00))
#define REG_RBR     (*(volatile uint32_t *)(MMUART1_BASE + 0x00))
#define REG_IER     (*(volatile uint32_t *)(MMUART1_BASE + 0x04))
#define REG_LCR     (*(volatile uint32_t *)(MMUART1_BASE + 0x0C))
#define REG_LSR     (*(volatile uint32_t *)(MMUART1_BASE + 0x14))

#define LSR_THRE    0x20  /* Transmitter Holding Register Empty */

void uart_init(void) {
    /* 8 data bits, 1 stop bit, no parity (8-N-1) */
    REG_LCR = 0x03;
    /* Enable Received Data Available (ERBFI, bit 0) interrupt for PLIC IRQ 91 */
    REG_IER = 0x01;
}

int uart_has_rx(void) {
    return (REG_LSR & 0x01); /* Bit 0: Data Ready */
}

char uart_getc(void) {
    return (char)(REG_RBR & 0xFF);
}

void uart_putc(char ch) {
    /* Poll Transmitter Holding Register Empty (THRE) bit */
    while ((REG_LSR & LSR_THRE) == 0);
    REG_THR = (uint32_t)ch;

    if (ch == '\n') {
        while ((REG_LSR & LSR_THRE) == 0);
        REG_THR = '\r';
    }
}

void uart_puts(const char *str) {
    if (!str) return;
    while (*str) {
        uart_putc(*str++);
    }
}

void uart_write(const char *data, size_t len) {
    if (!data || len == 0) return;
    for (size_t i = 0; i < len; i++) {
        uart_putc(data[i]);
    }
}
