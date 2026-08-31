/*
 * Copyright (c) 2026 Eclipse ThreadX contributors
 *
 * This program and the accompanying materials are made available
 * under the terms of the MIT license which is available at
 * https://opensource.org/licenses/MIT.
 *
 * SPDX-License-Identifier: MIT
 */

    .section .text._start
    .global  _start
    .align   4

_start:
    /* Disable interrupts on current hart */
    csrci   mstatus, 8

    /* Read machine hart ID (mhartid) */
    csrr    a0, mhartid
    li      t0, 1
    bne     a0, t0, .park_hart

    /* Setup stack pointer and global pointer for Hart 1 */
    .option push
    .option norelax
    la      gp, __global_pointer$
    .option pop
    la      sp, __stack_top

    /* Set Machine Trap-Vector Base Address to trap_entry */
    la      t0, trap_entry
    csrw    mtvec, t0

    /* Note: In Renode direct ELF loading, LMA == VMA so .data is already placed in DRAM.
     * On physical hardware booting via HSS, HSS loads sections before jumping to _start. */
    /* Zero-initialize .bss section */
    la      t0, __bss_start
    la      t1, __bss_end
.zero_bss_loop:
    bgeu    t0, t1, .bss_done
    sd      zero, 0(t0)
    addi    t0, t0, 8
    j       .zero_bss_loop
.bss_done:

    /* Jump to main application entry */
    call    main

.park_hart:
    /* Park unused harts in low-power WFI loop */
    wfi
    j       .park_hart
