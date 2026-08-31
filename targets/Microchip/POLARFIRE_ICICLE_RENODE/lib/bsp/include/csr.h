/*
 * Copyright (c) 2026 Eclipse ThreadX contributors
 *
 * This program and the accompanying materials are made available
 * under the terms of the MIT license which is available at
 * https://opensource.org/licenses/MIT.
 *
 * SPDX-License-Identifier: MIT
 */

#ifndef RISCV_CSR_H
#define RISCV_CSR_H

// Machine Status Register, mstatus
#define MSTATUS_MPP_MASK 	(3L << 11)   // previous mode.
#define MSTATUS_MPP_M 		(3L << 11)
#define MSTATUS_MPP_S 		(1L << 11)
#define MSTATUS_MPP_U 		(0L << 11)
#define MSTATUS_MIE 		(1L << 3)    // machine-mode interrupt enable.
#define MSTATUS_MPIE		(1L << 7)
#define MSTATUS_FS		(1L << 13)

// Machine-mode Interrupt Enable
#define MIE_MTIE		(1L << 7)
#define MIE_MSIE		(1L << 3)
#define MIE_MEIE		(1L << 11)
#define MIE_STIE 		(1L << 5)  // supervisor timer
#define MIE_SSIE		(1L << 1)
#define MIE_SEIE		(1L << 9)

#endif /* RISCV_CSR_H */
