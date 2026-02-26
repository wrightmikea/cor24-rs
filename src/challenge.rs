//! Challenge system for COR24 emulator

use crate::cpu::CpuState;

/// A challenge for the user to complete
#[derive(Clone)]
pub struct Challenge {
    pub id: usize,
    pub name: String,
    pub description: String,
    pub initial_code: String,
    pub hint: String,
    pub validator: fn(&CpuState) -> bool,
}

/// Get all available challenges
pub fn get_challenges() -> Vec<Challenge> {
    vec![
        Challenge {
            id: 1,
            name: "Load and Add".to_string(),
            description: "Load the value 10 into r0, then add 5 to it. Result should be 15 in r0."
                .to_string(),
            initial_code: "; Load 10 into r0, add 5\n; Result: r0 = 15\n\n".to_string(),
            hint: "Use 'lc r0,10' to load 10, then 'add r0,5' to add 5".to_string(),
            validator: |cpu| cpu.get_reg(0) == 15,
        },
        Challenge {
            id: 2,
            name: "Compare and Branch".to_string(),
            description: "Set r0 to 1 if 5 < 10 (signed), otherwise 0. Use cls and brt/brf."
                .to_string(),
            initial_code: "; Compare 5 < 10 and set r0 accordingly\n; Result: r0 = 1\n\n"
                .to_string(),
            hint: "Load values, use cls to compare, then mov r0,c to get the result".to_string(),
            validator: |cpu| cpu.get_reg(0) == 1,
        },
        Challenge {
            id: 3,
            name: "Stack Operations".to_string(),
            description: "Push values 1, 2, 3 onto the stack, then pop them into r0, r1, r2."
                .to_string(),
            initial_code: "; Push 1, 2, 3 then pop into r0, r1, r2\n; Result: r0=3, r1=2, r2=1\n\n"
                .to_string(),
            hint: "Remember LIFO order - last pushed is first popped".to_string(),
            validator: |cpu| cpu.get_reg(0) == 3 && cpu.get_reg(1) == 2 && cpu.get_reg(2) == 1,
        },
    ]
}

/// Get example programs
pub fn get_examples() -> Vec<(String, String, String)> {
    vec![
        (
            "Basic Arithmetic".to_string(),
            "Load values and perform arithmetic operations.".to_string(),
            r#"; Example 1: Basic Arithmetic
; Load constants and add them

        lc      r0,10       ; r0 = 10
        lc      r1,20       ; r1 = 20
        add     r0,r1       ; r0 = r0 + r1 = 30

        lc      r2,5        ; r2 = 5
        add     r0,r2       ; r0 = 35

        halt                ; Stop execution
"#
            .to_string(),
        ),
        (
            "Compare and Branch".to_string(),
            "Compare values and branch based on results.".to_string(),
            r#"; Example 2: Compare and Branch
; Compare two values and branch if less than

        lc      r0,5        ; r0 = 5
        lc      r1,10       ; r1 = 10

        cls     r0,r1       ; C = (r0 < r1) = true
        brt     less        ; Branch if true

        lc      r2,0        ; Not taken
        bra     done

less:   lc      r2,1        ; r2 = 1 (5 < 10)

done:   halt
"#
            .to_string(),
        ),
        (
            "Stack Frame".to_string(),
            "Set up a stack frame like a C function.".to_string(),
            r#"; Example 3: Stack Frame
; Simulate a C function prologue/epilogue

        ; Function entry
        push    fp          ; Save frame pointer
        push    r2          ; Save callee-saved reg
        push    r1          ; Save return address
        mov     fp,sp       ; Set up frame pointer

        ; Function body
        lc      r0,42       ; Return value

        ; Function exit
        mov     sp,fp       ; Restore stack
        pop     r1          ; Restore r1
        pop     r2          ; Restore r2
        pop     fp          ; Restore fp

        halt                ; (would be jmp (r1))
"#
            .to_string(),
        ),
        (
            "Loop Counter".to_string(),
            "Count from 0 to 5 using a loop.".to_string(),
            r#"; Example 4: Loop Counter
; Count from 0 to 5

        lc      r0,0        ; r0 = counter = 0
        lc      r1,5        ; r1 = limit = 5

loop:   add     r0,1        ; counter++
        cls     r0,r1       ; C = (counter < limit)
        brt     loop        ; Continue if less

        ; r0 = 5 when done
        halt
"#
            .to_string(),
        ),
        (
            "Memory Access".to_string(),
            "Store and load values from memory.".to_string(),
            r#"; Example 5: Memory Access
; Store values to memory and read them back

        lc      r0,100      ; Value to store
        lc      r1,0x1000   ; Memory address (high byte)

        ; Store byte
        sb      r0,0(r1)    ; mem[0x1000] = 100

        ; Load it back
        lb      r2,0(r1)    ; r2 = mem[0x1000]

        ; r2 should be 100
        halt
"#
            .to_string(),
        ),
    ]
}
