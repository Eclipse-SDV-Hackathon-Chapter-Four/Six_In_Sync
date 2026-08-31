/*
 * Copyright (c) 2026 Eclipse ThreadX contributors
 *
 * This program and the accompanying materials are made available
 * under the terms of the MIT license which is available at
 * https://opensource.org/licenses/MIT.
 *
 * SPDX-License-Identifier: MIT
 */

/*
 * Linker Script for Microchip PolarFire SoC Icicle Kit (Renode Execution)
 * Target Memory: LPDDR4 DRAM starting at 0x80000000 (1 GiB range)
 */

OUTPUT_ARCH("riscv")
ENTRY(_start)

MEMORY
{
    DRAM (rwx) : ORIGIN = 0x80000000, LENGTH = 0x40000000 /* 1 GiB System DRAM */
}

SECTIONS
{
    . = 0x80000000;

    .text : ALIGN(16)
    {
        *(.text._start)
        *(.text*)
        *(.rodata*)
    } > DRAM

    .data : ALIGN(16)
    {
        *(.data*)
    } > DRAM

    .sdata : ALIGN(8)
    {
        __global_pointer$ = . + 0x800;
        *(.srodata*)
        *(.sdata*)
        *(.gnu.linkonce.s.*)
    } > DRAM

    .sbss : ALIGN(8)
    {
        *(.sbss*)
        *(.gnu.linkonce.sb.*)
    } > DRAM

    .bss : ALIGN(16)
    {
        __bss_start = .;
        *(.bss*)
        *(.gnu.linkonce.b.*)
        *(COMMON)
        . = ALIGN(8);
        __bss_end = .;
    } > DRAM

    . = ALIGN(16);
    . = . + 0x4000; /* 16 KB Boot Stack */
    __stack_top = .;
    __end = .;
    _end = .;
}
