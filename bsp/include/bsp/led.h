/*
 * Copyright (c) 2026 Eclipse ThreadX contributors
 *
 * This program and the accompanying materials are made available
 * under the terms of the MIT license which is available at
 * https://opensource.org/licenses/MIT.
 *
 * SPDX-License-Identifier: MIT
 */

#ifndef BSP_LED_H
#define BSP_LED_H

/**
 * @brief Initialize the user LED hardware pin/peripherals.
 */
void bsp_led_init(void);

/**
 * @brief Turn the user LED on.
 */
void bsp_led_on(void);

/**
 * @brief Turn the user LED off.
 */
void bsp_led_off(void);

/**
 * @brief Toggle the state of the user LED.
 */
void bsp_led_toggle(void);

#endif /* BSP_LED_H */
