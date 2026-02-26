# COR24 ISA Reference

C-Oriented RISC 24-bit Instruction Set Architecture

## Overview

COR24 is a 24-bit RISC architecture designed for efficient C code execution. It features:

- **24-bit data path** - Registers and addresses are 24 bits wide
- **8 general-purpose registers** - r0 through r7
- **Single condition flag** - C (carry/condition)
- **Variable-length instructions** - 1 to 4 bytes
- **Little-endian** byte ordering
- **Memory-mapped I/O**

## Registers

| Register | Name | Description |
|----------|------|-------------|
| r0 | - | General purpose / return value |
| r1 | - | Return address (link register) |
| r2 | - | General purpose |
| r3 | fp | Frame pointer |
| r4 | sp | Stack pointer |
| r5 | z | Zero register (for comparisons) |
| r6 | - | Interrupt vector |
| r7 | - | Used for absolute jumps |
| PC | - | Program counter (24-bit) |
| C | - | Condition flag (1-bit) |

## Memory Map

| Address Range | Description |
|---------------|-------------|
| 0x000000 - 0xFDFFFF | SRAM (external memory) |
| 0xFE0000 - 0xFEFFFF | Embedded block RAM (fast) |
| 0xFF0000 - 0xFFFFFF | Memory-mapped I/O |

The CPU starts execution at address 0xFEE000 on reset.

## Instruction Set Summary

### Arithmetic Instructions

| Mnemonic | Bytes | Opcode | Description |
|----------|-------|--------|-------------|
| `add ra,rb` | 1 | 0x00 | ra = ra + rb |
| `add ra,dd` | 2 | 0x01 | ra = ra + sign_extend(dd) |
| `sub ra,rb` | 1 | 0x1A | ra = ra - rb |
| `sub sp,dddddd` | 4 | 0x1B | sp = sp - dddddd (24-bit) |
| `mul ra,rb` | 1 | 0x12 | ra = ra * rb (lower 24 bits) |

### Logical Instructions

| Mnemonic | Bytes | Opcode | Description |
|----------|-------|--------|-------------|
| `and ra,rb` | 1 | 0x02 | ra = ra & rb |
| `or ra,rb` | 1 | 0x13 | ra = ra \| rb |
| `xor ra,rb` | 1 | 0x1E | ra = ra ^ rb |

### Shift Instructions

| Mnemonic | Bytes | Opcode | Description |
|----------|-------|--------|-------------|
| `shl ra,rb` | 1 | 0x17 | ra = ra << rb[4:0] |
| `sra ra,rb` | 1 | 0x18 | ra = ra >> rb[4:0] (arithmetic) |
| `srl ra,rb` | 1 | 0x19 | ra = ra >> rb[4:0] (logical) |

### Compare Instructions

| Mnemonic | Bytes | Opcode | Description |
|----------|-------|--------|-------------|
| `ceq ra,rb` | 1 | 0x06 | C = (ra == rb) |
| `cls ra,rb` | 1 | 0x07 | C = (ra < rb) signed |
| `clu ra,rb` | 1 | 0x08 | C = (ra < rb) unsigned |

### Branch Instructions

| Mnemonic | Bytes | Opcode | Description |
|----------|-------|--------|-------------|
| `bra dd` | 2 | 0x03 | PC = PC + sign_extend(dd) |
| `brf dd` | 2 | 0x04 | if (!C) PC = PC + sign_extend(dd) |
| `brt dd` | 2 | 0x05 | if (C) PC = PC + sign_extend(dd) |

### Jump Instructions

| Mnemonic | Bytes | Opcode | Description |
|----------|-------|--------|-------------|
| `jmp (ra)` | 1 | 0x0A | PC = ra |
| `jal ra,(rb)` | 1 | 0x09 | ra = PC + 1; PC = rb |

### Load Instructions

| Mnemonic | Bytes | Opcode | Description |
|----------|-------|--------|-------------|
| `la ra,dddddd` | 4 | 0x0B | ra = dddddd (24-bit address) |
| `lb ra,dd(rb)` | 2 | 0x0C | ra = sign_extend(mem[rb + dd]) |
| `lbu ra,dd(rb)` | 2 | 0x0D | ra = zero_extend(mem[rb + dd]) |
| `lc ra,dd` | 2 | 0x0E | ra = sign_extend(dd) |
| `lcu ra,dd` | 2 | 0x0F | ra = zero_extend(dd) |
| `lw ra,dd(rb)` | 2 | 0x10 | ra = mem24[rb + dd] |

### Store Instructions

| Mnemonic | Bytes | Opcode | Description |
|----------|-------|--------|-------------|
| `sb ra,dd(rb)` | 2 | 0x16 | mem[rb + dd] = ra[7:0] |
| `sw ra,dd(rb)` | 2 | 0x1C | mem24[rb + dd] = ra |

### Move Instructions

| Mnemonic | Bytes | Opcode | Description |
|----------|-------|--------|-------------|
| `mov ra,rb` | 1 | 0x11 | ra = rb |
| `mov ra,c` | 1 | 0x11 | ra = C (when rb=5) |
| `sxt ra,rb` | 1 | 0x1D | ra = sign_extend(rb[7:0]) |
| `zxt ra,rb` | 1 | 0x1F | ra = zero_extend(rb[7:0]) |

### Stack Instructions

| Mnemonic | Bytes | Opcode | Description |
|----------|-------|--------|-------------|
| `push ra` | 1 | 0x15 | sp -= 3; mem24[sp] = ra |
| `pop ra` | 1 | 0x14 | ra = mem24[sp]; sp += 3 |

## Instruction Encoding

Instructions are encoded with a lookup ROM that maps 8-bit opcodes to decoded operations. The instruction byte encodes the operation and register operands together.

### Single-Byte Instructions (Register Operations)

| Byte | Instruction |
|------|-------------|
| 0x01 | add r0,r1 |
| 0x02 | add r0,r2 |
| 0x1A | cls r0,r2 |
| 0x1B | cls r1,r0 |
| 0x25 | jal r1,(r0) |
| 0x27 | jmp (r1) |
| 0x57 | mov r0,r2 |
| 0x65 | mov fp,sp |
| 0x69 | mov sp,fp |
| 0x7A | pop r1 |
| 0x7B | pop r2 |
| 0x7C | pop fp |
| 0x7D | push r0 |
| 0x7E | push r1 |
| 0x7F | push r2 |
| 0x80 | push fp |
| 0xCE | clu z,r0 |

### Two-Byte Instructions (Register + Immediate)

| First Byte | Instruction Format |
|------------|-------------------|
| 0x09 | add r0,imm8 |
| 0x0B | add r2,imm8 |
| 0x0C | add sp,imm8 |
| 0x13 | bra offset8 |
| 0x14 | brf offset8 |
| 0x15 | brt offset8 |
| 0x2C | lb r0,(r0) with offset |
| 0x2E | lb r0,(r2) with offset |
| 0x44 | lc r0,imm8 |
| 0x45 | lc r1,imm8 |
| 0x46 | lc r2,imm8 |
| 0x4D | lw r0,offset(fp) |
| 0x51 | lw r1,offset(fp) |
| 0x55 | lw r2,offset(fp) |
| 0x82 | sb r0,(r2) with offset |
| 0x84 | sb r1,(r0) with offset |
| 0xA6 | sw r0,offset(fp) |

### Four-Byte Instructions (Register + 24-bit Address)

| First Byte | Instruction Format |
|------------|-------------------|
| 0x29 | la r0,addr24 |
| 0x2A | la r1,addr24 |
| 0x2B | la r2,addr24 |

## Calling Convention

### Function Prologue
```asm
push    fp          ; Save frame pointer
push    r2          ; Save callee-saved register
push    r1          ; Save return address
mov     fp,sp       ; Set up frame pointer
add     sp,-N       ; Allocate stack space
```

### Function Epilogue
```asm
mov     sp,fp       ; Restore stack pointer
pop     r1          ; Restore return address
pop     r2          ; Restore callee-saved register
pop     fp          ; Restore frame pointer
jmp     (r1)        ; Return
```

### Argument Passing
- Arguments are pushed right-to-left onto the stack
- Return value is in r0
- Caller cleans up arguments with `add sp,N`

### Stack Frame Layout
```
Higher addresses
+----------------+
| Argument N     | fp + (N*3 + 6)
| ...            |
| Argument 1     | fp + 9
+----------------+
| Saved r1       | fp + 6
| Saved r2       | fp + 3
| Saved fp       | fp + 0
+----------------+
| Local var 1    | fp - 3
| Local var 2    | fp - 6
| ...            |
+----------------+
Lower addresses (sp)
```

## Word Size

- Words are 24 bits (3 bytes)
- Stack operations push/pop 3 bytes at a time
- Word addresses should be 3-byte aligned for efficiency

## Example: Fibonacci

```asm
_fib:
        push    fp              ; Save frame pointer
        push    r2              ; Save r2
        push    r1              ; Save return address
        mov     fp,sp           ; Set up frame
        add     sp,-3           ; Local variable space
        lw      r2,9(fp)        ; Load argument n

        lc      r0,2            ; Load constant 2
        cls     r2,r0           ; Compare n < 2
        brf     L17             ; Branch if false

        lc      r0,1            ; Return 1
        bra     L16             ; Jump to epilogue

L17:
        mov     r0,r2           ; r0 = n
        add     r0,-1           ; r0 = n - 1
        push    r0              ; Push argument
        la      r0,_fib         ; Load fib address
        jal     r1,(r0)         ; Call fib(n-1)
        add     sp,3            ; Clean up argument
        sw      r0,-3(fp)       ; Save result

        mov     r0,r2           ; r0 = n
        add     r0,-2           ; r0 = n - 2
        push    r0              ; Push argument
        la      r0,_fib         ; Load fib address
        jal     r1,(r0)         ; Call fib(n-2)
        add     sp,3            ; Clean up argument
        lw      r1,-3(fp)       ; Load fib(n-1)
        add     r0,r1           ; r0 = fib(n-1) + fib(n-2)

L16:
        mov     sp,fp           ; Restore stack
        pop     r1              ; Restore return address
        pop     r2              ; Restore r2
        pop     fp              ; Restore frame pointer
        jmp     (r1)            ; Return
```

## Interrupts

- Interrupt vector is in r6
- When an interrupt occurs, `jal r7,(r6)` is executed
- Return from interrupt with `jmp (r7)`
- Interrupts can be disabled by the `intis` flag

## References

- Verilog source: `cor24_cpu.v`
- Example programs: `demo/*.lst`
- Hardware manual: `doc/COR24-TB-MAN.pdf`
- MakerLisp: https://makerlisp.com

## License

The COR24 CPU is open-source under the MIT License.
