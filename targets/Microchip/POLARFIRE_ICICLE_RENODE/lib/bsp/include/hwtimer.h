/*
 * Copyright (c) 2026 Eclipse ThreadX contributors
 *
 * This program and the accompanying materials are made available
 * under the terms of the MIT license which is available at
 * https://opensource.org/licenses/MIT.
 *
 * SPDX-License-Identifier: MIT
 */

#ifndef POLARFIRE_HWTIMER_H
#define POLARFIRE_HWTIMER_H

#include <stdint.h>
#include "board_config.h"

#define CLINT_BASE           0x02000000ULL
#define MTIME_REG            (*(volatile uint64_t *)(CLINT_BASE + 0xBFF8))
#define HART1_MTIMECMP_REG   (*(volatile uint64_t *)(CLINT_BASE + 0x4008))

/* CLINT cycles between ThreadX system ticks. BSP_TICK_RATE_HZ lives in
 * board_config.h so this header stays independent of the ThreadX includes;
 * main.c carries a compile-time check that it still agrees with
 * TX_TIMER_TICKS_PER_SECOND, so the two cannot silently desync. */
#define TICK_CYCLES          (BSP_CLINT_RTC_FREQ_HZ / BSP_TICK_RATE_HZ)

void hwtimer_init(void);
void hwtimer_ack(void);

/**
 * @brief Compute the comparand for the next system tick.
 *
 * Normally this is one tick past the previous deadline. If the previous
 * deadline is already at or behind @p now, one or more ticks were missed and
 * re-arming relatively would leave the timer firing continuously until it
 * caught up, so the deadline is rebased onto @p now instead.
 *
 * Kept as a pure function of its arguments so the catch-up behaviour can be
 * verified without writing to the CLINT.
 *
 * @param current_cmp Comparand currently programmed into mtimecmp.
 * @param now         Current value of the shared mtime counter.
 * @return Comparand to program for the next tick.
 */
uint64_t hwtimer_next_cmp(uint64_t current_cmp, uint64_t now);

#endif /* POLARFIRE_HWTIMER_H */
