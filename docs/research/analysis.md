# COR24-TB ISA Emulator Development Analysis

## Executive Summary

The COR24-TB is a 24-bit RISC processor implemented on a Lattice MachXO2 FPGA. This document analyzes the available source files to identify resources useful for building an ISA emulator, strategies for implementation, and approaches for validation.

---

## 1. Source File Classification

### 1.1 Architecture Definition Files (Critical for Emulator)

| File | Description | Emulator Relevance |
|------|-------------|-------------------|
| `diamond/source/cor24_cpu.v` | Complete CPU implementation (1100 lines) | **PRIMARY** - Defines all 32 instructions, execution stages, register file, memory access patterns |
| `diamond/source/dis_rom/dis_rom_init.mem` | Instruction decode ROM (256 entries) | **CRITICAL** - Maps instruction bytes to decode values (opcode + register indices) |
| `diamond/source/cor24_io.v` | I/O module implementation | **ESSENTIAL** - Defines I/O port addresses, UART interface, interrupt logic |
| `diamond/source/spmul.v` | 24x24 bit multiplier | **REFERENCE** - Multi-cycle multiplication behavior |
| `diamond/source/add24ci/add24ci.v` | 24-bit adder with carry-in | **REFERENCE** - Address/operand addition |
| `diamond/source/add24cico/add24cico.v` | 24-bit adder with carry-in/out | **REFERENCE** - Comparison operations |

### 1.2 Memory System Files

| File | Description |
|------|-------------|
| `diamond/source/ebr_dpram_true/ebr_dpram_true.v` | Dual-port embedded block RAM (4KB) |
| `diamond/source/ebr_dpram_true/ebr_rom_init.mem` | Boot ROM initialization data |
| `diamond/source/tb_cor24.v` | SRAM simulation model (128KB, 2-cycle byte-wide) |

### 1.3 Peripheral Implementation Files

| File | Description |
|------|-------------|
| `diamond/source/uart_rx.v` | UART receiver (8N1, 921600 baud, 4-entry FIFO) |
| `diamond/source/uart_tx.v` | UART transmitter |
| `diamond/source/baudrates.v` | Baud rate timing constants |

### 1.4 C Source Code Examples

| File | Description | Validation Use |
|------|-------------|----------------|
| `demo/hello.c` | Basic printf test | Quick sanity check |
| `demo/fib.c` | Recursive Fibonacci (n=33) | Stack/recursion validation |
| `demo/blinky.c` | LED follows button | I/O port validation |
| `demo/memtest.c` | Comprehensive memory test | Memory subsystem validation |
| `demo/sieve.c` | Sieve of Eratosthenes | Array/loop operations |
| `demo/pidemo.c` | PI approximation (100K iterations) | Floating-point/math |
| `demo/matrix.c` | 3x3 double matrix multiplication | Complex arithmetic |
| `demo/uartcint.c` | UART interrupt handler | Interrupt system validation |
| `demo/loadngo.c` | Monitor/bootloader program | UART + jump-to-address capability |
| `demo/tprintf.c` | Printf format testing | String formatting |

### 1.5 Assembly Source Code

| File | Description |
|------|-------------|
| `demo/uartaint.s` | UART interrupt service routine in assembly |

**Key details from `uartaint.s`:**
- Shows register save/restore conventions (push fp, r2, r1, r0, then carry flag)
- Demonstrates calling convention for assembly-to-C transitions
- Shows interrupt return via `jmp (ir)` where `ir` = R7
- Shows `iv` (R6) is the interrupt vector register

### 1.6 Listing Files (Assembled Output)

These are invaluable for verifying instruction encoding:

| File | Contents |
|------|----------|
| `demo/hello.lst` | Compiled hello.c with hex opcodes |
| `demo/fib.lst` | Compiled fib.c |
| `demo/memtest.lst` | Compiled memtest.c |
| `demo/blinky.lst` | Compiled blinky.c |
| `demo/loadngo.lst` | Compiled monitor program |
| `demo/uartintr.lst` | Compiled UART interrupt test |

### 1.7 Binary/Memory Image Files

| File | Description |
|------|-------------|
| `demo/loadngo.mem` | Hex memory dump of monitor program |
| `demo/*.lgo` | Load-and-go format files |

### 1.8 Documentation

| File | Description |
|------|-------------|
| `doc/COR24-TB-MAN.pdf` | Complete user manual (ISA, registers, memory map) |
| `doc/COR24-TB-SCH.pdf` | Hardware schematic |
| `doc/COR24-TB-BOM.pdf` | Bill of materials |
| `doc/RELEASE.txt` | Release notes and version info |

### 1.9 Host Tools

| File | Description |
|------|-------------|
| `tools/te.c` | Terminal emulator for host-to-board communication |

---

## 2. Architecture Specification

### 2.1 Register Architecture

Extracted from `cor24_cpu.v`:

```
Registers:
  R0-R7    : 24-bit general purpose registers (RF[7:0])
  PC       : 24-bit program counter
  C        : 1-bit carry/condition flag

Conventional Usage (from assembly code analysis):
  R3 (FP)  : Frame pointer
  R4 (SP)  : Stack pointer (initialized to 0xFEEC00 on reset)
  R6 (IV)  : Interrupt vector address
  R7 (IR)  : Return address / interrupt return
  R5       : Can reference carry flag in MOV instruction
```

### 2.2 Memory Map

Extracted from `cor24_cpu.v` macros:

```
Address Range         Description
--------------------- ------------------------------------
0x000000 - 0xFDFFFF   External SRAM (up to 16MB addressable, 512KB on board)
0xFEE000 - 0xFEFFFF   Embedded Block RAM (4KB instruction/data)
0xFF0000 - 0xFFFFFF   I/O Space

Reset Vector: 0xFEE000 (boot from embedded RAM)
Initial SP:   0xFEEC00
```

### 2.3 I/O Port Map

Extracted from `cor24_io.v`:

```
Port Address    R/W   Description
--------------- ----- ------------------------------------------
0xFF0000        R     Button input (bit 0)
0xFF0000        W     LED output (bit 0)
0xFF0010        R/W   Interrupt enable register (bit 0 = UART RX)
0xFF0100        R     UART receive data (auto-acknowledge)
0xFF0100        W     UART transmit data
0xFF0101        R     UART status register:
                        bit 7: TX busy
                        bit 2: RX overflow
                        bit 1: CTS active
                        bit 0: RX data ready
```

### 2.4 Instruction Set

32 instructions extracted from `cor24_cpu.v` (5-bit opcode 0x00-0x1F):

| Opcode | Mnemonic | Format | Description |
|--------|----------|--------|-------------|
| 0x00 | `add ra,rb` | 1-byte | ra = ra + rb |
| 0x01 | `add ra,dd` | 2-byte | ra = ra + sign_extend(dd) |
| 0x02 | `and ra,rb` | 1-byte | ra = ra & rb |
| 0x03 | `bra dd` | 2-byte | PC = PC + sign_extend(dd) |
| 0x04 | `brf dd` | 2-byte | if (!C) PC = PC + sign_extend(dd) |
| 0x05 | `brt dd` | 2-byte | if (C) PC = PC + sign_extend(dd) |
| 0x06 | `ceq ra,rb` | 1-byte | C = (ra == rb) |
| 0x07 | `cls ra,rb` | 1-byte | C = (ra < rb) signed |
| 0x08 | `clu ra,rb` | 1-byte | C = (ra < rb) unsigned |
| 0x09 | `jal ra,(rb)` | 1-byte | ra = PC+1; PC = rb |
| 0x0A | `jmp (ra)` | 1-byte | PC = ra; if ra=r7, clear interrupt |
| 0x0B | `la ra,dddddd` | 4-byte | ra = 24-bit immediate; if ra=r7, jump |
| 0x0C | `lb ra,dd(rb)` | 2-byte | ra = sign_extend(mem[rb + sign_extend(dd)]) |
| 0x0D | `lbu ra,dd(rb)` | 2-byte | ra = zero_extend(mem[rb + sign_extend(dd)]) |
| 0x0E | `lc ra,dd` | 2-byte | ra = sign_extend(dd) |
| 0x0F | `lcu ra,dd` | 2-byte | ra = zero_extend(dd) |
| 0x10 | `lw ra,dd(rb)` | 2-byte | ra = mem24[rb + sign_extend(dd)] |
| 0x11 | `mov ra,rb` | 1-byte | ra = rb; if rb=r5, ra = {0,C} |
| 0x12 | `mul ra,rb` | 1-byte | ra = (ra * rb)[23:0] (24 cycles) |
| 0x13 | `or ra,rb` | 1-byte | ra = ra | rb |
| 0x14 | `pop ra` | 1-byte | ra = mem24[sp]; sp = sp + 3 |
| 0x15 | `push ra` | 1-byte | sp = sp - 3; mem24[sp] = ra |
| 0x16 | `sb ra,dd(rb)` | 2-byte | mem[rb + sign_extend(dd)] = ra[7:0] |
| 0x17 | `shl ra,rb` | 1-byte | ra = ra << rb[4:0] |
| 0x18 | `sra ra,rb` | 1-byte | ra = ra >>> rb[4:0] (arithmetic) |
| 0x19 | `srl ra,rb` | 1-byte | ra = ra >> rb[4:0] (logical) |
| 0x1A | `sub ra,rb` | 1-byte | ra = ra - rb |
| 0x1B | `sub sp,dddddd` | 4-byte | sp = sp - 24-bit immediate |
| 0x1C | `sw ra,dd(rb)` | 2-byte | mem24[rb + sign_extend(dd)] = ra |
| 0x1D | `sxt ra,rb` | 1-byte | ra = sign_extend(rb[7:0]) |
| 0x1E | `xor ra,rb` | 1-byte | ra = ra ^ rb |
| 0x1F | `zxt ra,rb` | 1-byte | ra = zero_extend(rb[7:0]) |

### 2.5 Instruction Encoding

From `dis_rom_init.mem` and listing files:

```
Single-byte instructions (opcode only):
  Byte format: OOOOO_RRR (O=opcode[4:0], R=ra[2:0] or encoding bits)

  For register-register ops: first byte encodes opcode+ra, rb implicit or same

Two-byte instructions (opcode + immediate):
  Byte 1: OOOOO_RRR (opcode + register)
  Byte 2: dd (8-bit signed/unsigned immediate)

Four-byte instructions (la, sub sp):
  Byte 1: OOOOO_RRR
  Bytes 2-4: 24-bit immediate (little-endian: low, mid, high)
```

**Example from hello.lst:**
```
000004 29 00 00 00      la      r0,__iob      ; 0x29 = opcode 0x0B (la), ra=r0
000008 09 06            add     r0,6          ; 0x09 = opcode 0x01 (add imm), ra=r0
00000a 7d               push    r0            ; 0x7D = opcode 0x15 (push), ra=r0
000014 25               jal     r1,(r0)       ; 0x25 = opcode 0x09 (jal), ra=r1
```

### 2.6 Interrupt Handling

From `cor24_cpu.v` and `cor24_io.v`:

```
1. Interrupt request generated when UART RX ready AND interrupt enable set
2. CPU checks irqis = intreq && !intis (not already in service)
3. Automatic JAL r7,(r6) issued - saves return address to R7, jumps to address in R6
4. ISR executes, ends with JMP (r7) to return
5. JMP to r7 clears intis flag, re-enabling interrupts
```

---

## 3. Hardware Interface Details

### 3.1 UART Implementation

From `uart_rx.v` and `uart_tx.v`:

```
Configuration:
  - 8N1 format (8 data bits, no parity, 1 stop bit)
  - 921600 baud (at 101.6064 MHz system clock)
  - Bit period: 110 clock cycles
  - Sample point: cycle 55 (mid-bit)

RX Features:
  - 4-entry circular buffer (indices ridx, widx)
  - Hardware RTS flow control (asserted when busy or data ready)
  - Overflow detection flag

TX Features:
  - Single-byte shift register
  - CTS flow control input
  - Busy flag while transmitting
```

### 3.2 GPIO (LED/Button)

From `cor24_io.v`:

```
Port 0xFF0000:
  Read:  bit 0 = button state (directly sampled)
  Write: bit 0 = LED state (active low on hardware)
```

### 3.3 Interrupt Controller

From `cor24_io.v`:

```
Port 0xFF0010:
  Read:  bit 0 = interrupt enable state
  Write: bit 0 = set interrupt enable (1=enabled)

Interrupt Sources:
  - UART RX data ready (only source in current implementation)
```

---

## 4. Emulator Implementation Strategy

### 4.1 Core Components

1. **Register File Module**
   ```
   - R[0-7]: 24-bit registers
   - PC: 24-bit program counter
   - C: 1-bit carry flag
   - SP convention: R4
   - FP convention: R3
   - IV convention: R6
   - Return addr: R7
   ```

2. **Memory Subsystem**
   ```
   - Unified 24-bit address space
   - Region detection: SRAM (< 0xFE0000), EBR (0xFExxxx), I/O (0xFFxxxx)
   - Byte-addressable with 24-bit word operations
   - Little-endian word storage
   ```

3. **Instruction Decoder**
   ```
   - Fetch byte, decode via lookup table or direct opcode extraction
   - Variable-length instruction handling (1, 2, or 4 bytes)
   - Handle special cases: la r7 = jump, mov ra,c, jmp r7 = iret
   ```

4. **Execution Engine**
   ```
   - State machine or direct execution model
   - Multi-cycle operations: MUL (24 cycles in hardware), shifts (1 per bit)
   - Memory access timing (optional for cycle-accurate emulation)
   ```

5. **I/O Subsystem**
   ```
   - UART simulation with virtual serial port
   - GPIO state (button input, LED output)
   - Interrupt controller with single source
   ```

### 4.2 Implementation Approaches

**Approach A: Interpreter (Recommended for Development)**
- Fetch-decode-execute loop
- Easy to debug and modify
- Good for initial development and validation

**Approach B: Cycle-Accurate Simulator**
- Model multi-stage pipeline from Verilog
- Match hardware timing exactly
- Useful for timing-sensitive software

**Approach C: JIT/Binary Translation**
- Translate COR24 instructions to host machine code
- Maximum performance
- Complex to implement correctly

### 4.3 Suggested Implementation Order

1. **Phase 1: Basic Execution**
   - Register file and ALU operations
   - Load/store byte and word
   - Branch instructions
   - JAL/JMP (no interrupts yet)
   - Run `hello.c` binary

2. **Phase 2: Full Instruction Set**
   - Multiplication (can simplify to single-cycle)
   - Shifts (all three types)
   - All compare operations
   - Push/pop stack operations
   - Run `fib.c` binary

3. **Phase 3: I/O and Peripherals**
   - UART TX (output to console)
   - UART RX (input from console)
   - LED/button ports
   - Run `blinky.c`, `loadngo.c`

4. **Phase 4: Interrupts**
   - Interrupt controller logic
   - Automatic JAL r7,(r6) on interrupt
   - JMP r7 = return from interrupt
   - Run `uartcint.c`

5. **Phase 5: Validation and Polish**
   - Run full test suite
   - Compare against hardware behavior
   - Optimize performance

---

## 5. Validation Strategy

### 5.1 Unit Tests (Instruction-Level)

Create test cases for each instruction:

```
Test Category              Instructions
-------------------------- ---------------------------
Arithmetic                 add, sub, mul
Logical                    and, or, xor
Shifts                     shl, sra, srl
Comparisons                ceq, cls, clu
Memory - Byte              lb, lbu, sb
Memory - Word              lw, sw
Stack                      push, pop
Control Flow               bra, brf, brt, jmp, jal, la
Data Movement              mov, lc, lcu, sxt, zxt
Special                    sub sp,imm
```

### 5.2 Integration Tests (Program-Level)

| Test Program | What It Validates |
|--------------|-------------------|
| `hello.c` | Function calls, stack, UART output, string handling |
| `fib.c` (n=33 → 5702887) | Recursion, stack depth, arithmetic |
| `blinky.c` | I/O ports, infinite loop |
| `memtest.c` | Memory addressing, data patterns |
| `sieve.c` | Arrays, loops, conditionals |
| `loadngo.c` | UART I/O, hex parsing, computed jumps |
| `uartcint.c` | Interrupts, ISR handling, circular buffers |

### 5.3 Validation Against Hardware

1. **Using the Terminal Emulator (`te.c`)**
   - Connect to real COR24-TB board via USB serial
   - Run same test programs on hardware and emulator
   - Compare outputs character-by-character

2. **Using the Monitor Program**
   - Load `loadngo` on hardware
   - Use L command to load test code
   - Use G command to execute
   - Compare results with emulator

3. **Trace Comparison**
   - Add trace output to emulator (PC, registers after each instruction)
   - Compare with Verilog simulation output (`$display` in `cor24_cpu.v`)

### 5.4 Test Vectors from Listing Files

The `.lst` files provide exact instruction sequences with addresses:

```
From hello.lst:
Address  Bytes           Instruction
-------- --------------- ----------------
000000   80              push    fp
000001   7f              push    r2
000002   7e              push    r1
000003   65              mov     fp,sp
000004   29 00 00 00     la      r0,__iob
...
```

These can be used to:
- Verify instruction decoding
- Create minimal test sequences
- Debug instruction-by-instruction

---

## 6. Key Implementation Details from Verilog

### 6.1 Sign Extension

From `cor24_cpu.v`:
```verilog
`define SGN16(x) {16{x[7]}}
assign seBPC = {`SGN16(BPC), BPC};
```
Sign extend 8-bit value to 24 bits by replicating bit 7.

### 6.2 Comparison Implementation

**CEQ (compare equal):**
```verilog
C <= !(RA ^ RB);  // C = 1 if RA == RB
```

**CLS (compare less signed):**
```verilog
// Uses subtraction and sign bit analysis
C <= (op1gpadd[23] && sumgpadd[23]) ||
     (op1gpadd[23] && !rtmp[23]) ||
     (sumgpadd[23] && !rtmp[23]);
```

**CLU (compare less unsigned):**
```verilog
// Uses RB + ~RA, checks carry out
C <= cogpadd;  // Carry out from RB + ~RA
```

### 6.3 Stack Operations

**PUSH:**
```verilog
// sp = sp - 3, then store word at sp
op2gpadd <= 24'hfffffd;  // -3
// Store rtmp (source register) at address
```

**POP:**
```verilog
// Load word from sp, then sp = sp + 3
// op2gpadd <= 24'h000003 after load
```

### 6.4 Interrupt Handling

```verilog
/* Interrupt entry - automatic JAL r7,(r6) */
`define INTJAL 11'h27e
if (irqis) begin
    intis <= 1'b1;
    ifbdec <= `INTJAL;  // Decode value for JAL r7,(r6)
    rjmp <= RF[6];      // Jump to interrupt vector
    // ... save PC to r7
end

/* Interrupt return - JMP (r7) clears intis */
if (&racode) begin  // ra == r7
    intis <= 1'b0;
end
```

### 6.5 Memory Timing

```
Embedded Block RAM (0xFExxxx):
  - 2 cycles from PC load to first byte
  - Word access: 4+ cycles (read 3 bytes sequentially)

SRAM (0x000000-0xFDFFFF):
  - 2-cycle access per byte
  - Word access: 6+ cycles

I/O (0xFFxxxx):
  - Variable timing based on device
  - UART RX: 2 cycles (wait for queue)
  - UART TX: 2 cycles (wait for not busy)
```

---

## 7. Files Summary by Priority

### Must-Have for Emulator Development

1. `diamond/source/cor24_cpu.v` - Complete ISA reference
2. `diamond/source/cor24_io.v` - I/O behavior
3. `diamond/source/dis_rom/dis_rom_init.mem` - Decode table
4. `demo/hello.lst` - Instruction encoding examples
5. `doc/COR24-TB-MAN.pdf` - Official documentation

### Highly Useful

1. `demo/*.c` - All C source files for test programs
2. `demo/*.lst` - All listing files for encoding reference
3. `diamond/source/uart_rx.v` - UART receive behavior
4. `diamond/source/uart_tx.v` - UART transmit behavior
5. `demo/uartaint.s` - Assembly calling conventions

### Reference Material

1. `diamond/source/tb_cor24.v` - Testbench with SRAM model
2. `diamond/source/spmul.v` - Multiplier timing
3. `diamond/source/baudrates.v` - UART timing constants
4. `tools/te.c` - Host communication tool

---

## 8. Recommended Next Steps

1. **Read the PDF manual** (`doc/COR24-TB-MAN.pdf`) for any details not in Verilog

2. **Extract the decode ROM** to create instruction decoder lookup table

3. **Start with `hello.c`** - implement minimal instruction subset to run it

4. **Use listing files** to verify instruction encoding during development

5. **Build UART console I/O first** - most test programs output via UART

6. **Save interrupts for last** - most programs don't require them initially

7. **Consider building a disassembler** as a byproduct - useful for debugging

---

## Appendix A: Quick Reference Card

```
Registers: R0-R7 (24-bit), PC (24-bit), C (1-bit)
Conventions: SP=R4, FP=R3, IV=R6, RET=R7

Memory Map:
  0x000000-0xFDFFFF  SRAM
  0xFEE000-0xFEFFFF  Boot ROM
  0xFF0000           LED/Button
  0xFF0010           Interrupt Enable
  0xFF0100           UART Data
  0xFF0101           UART Status

Instruction Formats:
  1-byte: [opcode:5][ra:3]
  2-byte: [opcode:5][ra:3] [imm8]
  4-byte: [opcode:5][ra:3] [imm8:low] [imm8:mid] [imm8:high]

Key Behaviors:
  - Little-endian word storage
  - Sign extension on byte loads (lb) and immediate loads (lc, add imm)
  - MUL returns low 24 bits of 48-bit product
  - Shifts use only low 5 bits of shift amount
  - JMP (r7) returns from interrupt
  - LA r7,addr is a direct jump (no link)
```
