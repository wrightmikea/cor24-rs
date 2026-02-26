//! Yew application for COR24 Assembly Emulator

use components::{
    Header, LegendItem, MemoryViewer, Modal, ProgramArea, Register, RegisterPanel, Sidebar,
    SidebarButton,
};
use yew::prelude::*;

use crate::challenge::{get_challenges, get_examples};
use crate::wasm::{WasmCpu, validate_challenge};

#[function_component(App)]
pub fn app() -> Html {
    // State management
    let cpu = use_state(WasmCpu::new);
    let program_code = use_state(|| String::from(EXAMPLE_PROGRAM));
    let assembly_output = use_state(|| None::<Html>);
    let assembly_lines = use_state(Vec::<String>::new);
    let last_registers = use_state(|| vec![0u32; 8]);
    let challenge_mode = use_state(|| false);
    let current_challenge_id = use_state(|| None::<usize>);
    let challenge_result = use_state(|| None::<Result<String, String>>);

    // Modal states
    let tutorial_open = use_state(|| false);
    let examples_open = use_state(|| false);
    let challenges_open = use_state(|| false);
    let isa_ref_open = use_state(|| false);
    let help_open = use_state(|| false);

    // Callbacks for modals
    let close_tutorial = {
        let tutorial_open = tutorial_open.clone();
        Callback::from(move |_| tutorial_open.set(false))
    };
    let close_examples = {
        let examples_open = examples_open.clone();
        Callback::from(move |_| examples_open.set(false))
    };
    let close_challenges = {
        let challenges_open = challenges_open.clone();
        Callback::from(move |_| challenges_open.set(false))
    };
    let close_isa_ref = {
        let isa_ref_open = isa_ref_open.clone();
        Callback::from(move |_| isa_ref_open.set(false))
    };
    let close_help = {
        let help_open = help_open.clone();
        Callback::from(move |_| help_open.set(false))
    };

    // Sidebar buttons with inline callbacks
    let sidebar_buttons = vec![
        SidebarButton {
            emoji: "📚".to_string(),
            label: "Tutorial".to_string(),
            onclick: {
                let tutorial_open = tutorial_open.clone();
                Callback::from(move |_| tutorial_open.set(true))
            },
            title: Some("Learn COR24 basics".to_string()),
        },
        SidebarButton {
            emoji: "📝".to_string(),
            label: "Examples".to_string(),
            onclick: {
                let examples_open = examples_open.clone();
                Callback::from(move |_| examples_open.set(true))
            },
            title: Some("Load example programs".to_string()),
        },
        SidebarButton {
            emoji: "🎯".to_string(),
            label: "Challenges".to_string(),
            onclick: {
                let challenges_open = challenges_open.clone();
                Callback::from(move |_| challenges_open.set(true))
            },
            title: Some("Test your skills".to_string()),
        },
        SidebarButton {
            emoji: "📖".to_string(),
            label: "ISA Ref".to_string(),
            onclick: {
                let isa_ref_open = isa_ref_open.clone();
                Callback::from(move |_| isa_ref_open.set(true))
            },
            title: Some("Instruction reference".to_string()),
        },
        SidebarButton {
            emoji: "❓".to_string(),
            label: "Help".to_string(),
            onclick: {
                let help_open = help_open.clone();
                Callback::from(move |_| help_open.set(true))
            },
            title: Some("Usage help".to_string()),
        },
    ];

    // CPU operation callbacks
    let on_assemble = {
        let cpu = cpu.clone();
        let assembly_output = assembly_output.clone();
        let assembly_lines = assembly_lines.clone();
        let program_code = program_code.clone();

        Callback::from(move |code: String| {
            program_code.set(code.clone());

            // Assemble the source code
            let mut new_cpu = (*cpu).clone();
            match new_cpu.assemble(&code) {
                Ok(_output) => {
                    // Get assembled lines for display
                    let lines = new_cpu.get_assembled_lines();
                    assembly_lines.set(lines);
                    cpu.set(new_cpu);

                    assembly_output.set(Some(html! {
                        <div class="success-text">
                            {"✓ Program assembled successfully"}
                        </div>
                    }));
                }
                Err(e) => {
                    assembly_lines.set(Vec::new());
                    assembly_output.set(Some(html! {
                        <div class="error-text">
                            {format!("Assembly error: {:?}", e)}
                        </div>
                    }));
                }
            }
        })
    };

    let on_step = {
        let cpu = cpu.clone();
        let assembly_output = assembly_output.clone();
        let last_registers = last_registers.clone();

        Callback::from(move |()| {
            let mut new_cpu = (*cpu).clone();

            // Save current state for change tracking
            last_registers.set(new_cpu.get_registers());

            match new_cpu.step() {
                Ok(_) => {
                    cpu.set(new_cpu);
                }
                Err(e) => {
                    assembly_output.set(Some(html! {
                        <div class="error-text">
                            {format!("Error: {:?}", e)}
                        </div>
                    }));
                }
            }
        })
    };

    let on_run = {
        let cpu = cpu.clone();
        let assembly_output = assembly_output.clone();

        Callback::from(move |()| {
            let mut new_cpu = (*cpu).clone();
            match new_cpu.run() {
                Ok(()) => {
                    cpu.set(new_cpu);
                    assembly_output.set(Some(html! {
                        <div class="success-text">
                            {"✓ Program completed"}
                        </div>
                    }));
                }
                Err(e) => {
                    assembly_output.set(Some(html! {
                        <div class="error-text">
                            {format!("Error: {:?}", e)}
                        </div>
                    }));
                }
            }
        })
    };

    let on_reset = {
        let cpu = cpu.clone();
        let assembly_output = assembly_output.clone();
        let assembly_lines = assembly_lines.clone();

        Callback::from(move |()| {
            // Full reset - create new CPU with cleared memory
            cpu.set(WasmCpu::new());
            assembly_lines.set(Vec::new());
            assembly_output.set(None);
        })
    };

    // Register panel data
    let registers = {
        let regs = (*cpu).get_registers();
        let last_regs = &*last_registers;
        let reg_names = ["r0", "r1", "r2", "fp", "sp", "z", "iv", "ir"];
        let mut reg_list = Vec::new();
        for i in 0..8 {
            let changed = regs[i] != last_regs[i];
            reg_list.push(Register {
                name: reg_names[i].to_string(),
                value: format!("0x{:06X} ({})", regs[i], regs[i] as i32),
                changed,
            });
        }
        reg_list
    };

    let legend_items = vec![
        LegendItem {
            label: "fp".to_string(),
            value: "Frame Pointer (r3)".to_string(),
            changed: false,
        },
        LegendItem {
            label: "sp".to_string(),
            value: "Stack Pointer (r4)".to_string(),
            changed: false,
        },
        LegendItem {
            label: "z".to_string(),
            value: "Zero Register (r5)".to_string(),
            changed: false,
        },
        LegendItem {
            label: "iv".to_string(),
            value: "Interrupt Vector (r6)".to_string(),
            changed: false,
        },
        LegendItem {
            label: "ir".to_string(),
            value: "Interrupt Return (r7)".to_string(),
            changed: false,
        },
        LegendItem {
            label: "C".to_string(),
            value: format!("Condition: {}", if (*cpu).get_c_flag() { "1" } else { "0" }),
            changed: false,
        },
    ];

    // Memory data
    let memory = (*cpu).get_memory_slice(0, 128);
    let pc = (*cpu).pc() as u16;

    // Get examples for the modal
    let examples = get_examples();

    html! {
        <div class="container">
            <Header title="COR24 C-Oriented RISC Assembly Emulator" />

            <Sidebar buttons={sidebar_buttons} />

            <div class="main-content">
                <ProgramArea
                    on_assemble={on_assemble}
                    on_step={on_step}
                    on_run={on_run}
                    on_reset={on_reset}
                    assembly_output={
                        if !assembly_lines.is_empty() {
                            // Show highlighted assembly lines
                            let pc = (*cpu).pc();
                            Some(html! {
                                <div>
                                    {for assembly_lines.iter().map(|line| {
                                        // Parse address from "ADDR: BYTES SOURCE" format
                                        let is_current = if line.len() > 4 && line.chars().nth(4) == Some(':') {
                                            if let Ok(addr) = u32::from_str_radix(&line[0..4], 16) {
                                                addr == pc
                                            } else {
                                                false
                                            }
                                        } else {
                                            false
                                        };

                                        let class = if is_current {
                                            "assembly-line current"
                                        } else {
                                            "assembly-line"
                                        };

                                        html! {
                                            <div class={class}>{line}</div>
                                        }
                                    })}
                                </div>
                            })
                        } else {
                            // Show success/error messages
                            (*assembly_output).clone()
                        }
                    }
                    initial_code={Some((*program_code).clone())}
                    step_enabled={!(*cpu).is_halted()}
                    run_enabled={!(*cpu).is_halted()}
                />

                <div class="right-panels">
                    <div class="registers-panel">
                        <RegisterPanel
                            registers={registers}
                            legend_items={legend_items}
                        />

                        // CPU Status
                        <div class="cpu-status">
                            <div class="status-item">
                                <span class="status-label">{"PC:"}</span>
                                <span class="status-value">{format!("0x{:06X}", (*cpu).pc())}</span>
                            </div>
                            <div class="status-item">
                                <span class="status-label">{"Cycles:"}</span>
                                <span class="status-value">{(*cpu).cycle_count()}</span>
                            </div>
                            <div class="status-item">
                                <span class="status-label">{"Instructions:"}</span>
                                <span class="status-value">{(*cpu).instruction_count()}</span>
                            </div>
                            <div class="status-item">
                                <span class="status-label">{"Status:"}</span>
                                <span class="status-value">
                                    {if (*cpu).is_halted() { "HALTED" } else { "RUNNING" }}
                                </span>
                            </div>
                        </div>
                    </div>

                    <MemoryViewer
                        memory={memory}
                        pc={pc}
                        title={Some("Memory (First 128 Bytes)".to_string())}
                        bytes_per_row={16}
                        bytes_to_show={128}
                    />
                </div>
            </div>

            // Challenge Mode Banner
            if *challenge_mode {
                if let Some(challenge_id) = *current_challenge_id {
                    <div class="challenge-banner">
                        <span class="challenge-indicator">{"⚡"}</span>
                        <span class="challenge-text">
                            {format!("Challenge Mode - Challenge {}", challenge_id)}
                        </span>
                        <button
                            class="check-button"
                            onclick={
                                let challenge_result = challenge_result.clone();
                                let program_code = program_code.clone();
                                Callback::from(move |_| {
                                    match validate_challenge(challenge_id, &(*program_code)) {
                                        Ok(passed) => {
                                            if passed {
                                                challenge_result.set(Some(Ok(format!("✅ Challenge {} PASSED!", challenge_id))));
                                            } else {
                                                challenge_result.set(Some(Err(format!("❌ Challenge {} did not pass. Check your solution.", challenge_id))));
                                            }
                                        }
                                        Err(e) => {
                                            challenge_result.set(Some(Err(format!("Validation error: {:?}", e))));
                                        }
                                    }
                                })
                            }
                        >
                            {"Check Solution"}
                        </button>
                        <button
                            class="exit-button"
                            onclick={
                                let challenge_mode = challenge_mode.clone();
                                let current_challenge_id = current_challenge_id.clone();
                                let challenge_result = challenge_result.clone();
                                Callback::from(move |_| {
                                    challenge_mode.set(false);
                                    current_challenge_id.set(None);
                                    challenge_result.set(None);
                                })
                            }
                        >
                            {"Exit"}
                        </button>
                    </div>
                }
            }

            // Success/Error Banners
            {
                if let Some(result) = &*challenge_result {
                    match result {
                        Ok(message) => html! {
                            <div class="success-banner">
                                <span class="banner-content">{message}</span>
                                <button
                                    class="dismiss-button"
                                    onclick={
                                        let challenge_result = challenge_result.clone();
                                        Callback::from(move |_| challenge_result.set(None))
                                    }
                                >
                                    {"×"}
                                </button>
                            </div>
                        },
                        Err(message) => html! {
                            <div class="error-banner">
                                <span class="banner-content">{message}</span>
                                <button
                                    class="dismiss-button"
                                    onclick={
                                        let challenge_result = challenge_result.clone();
                                        Callback::from(move |_| challenge_result.set(None))
                                    }
                                >
                                    {"×"}
                                </button>
                            </div>
                        }
                    }
                } else {
                    html! {}
                }
            }

            // Modals
            <Modal id="tutorial" title="Tutorial" active={*tutorial_open} on_close={close_tutorial}>
                {html! { <div>{TUTORIAL_CONTENT}</div> }}
            </Modal>

            <Modal id="examples" title="Examples" active={*examples_open} on_close={close_examples}>
                <div class="examples-list">
                    {for examples.iter().enumerate().map(|(idx, (title, desc, code))| {
                        let program_code = program_code.clone();
                        let examples_open = examples_open.clone();
                        let cpu = cpu.clone();
                        let assembly_output = assembly_output.clone();
                        let assembly_lines = assembly_lines.clone();
                        let code = code.clone();

                        let load_example = Callback::from(move |_: MouseEvent| {
                            // Reset CPU
                            cpu.set(WasmCpu::new());
                            assembly_output.set(None);
                            assembly_lines.set(Vec::new());

                            // Load new code
                            program_code.set(code.clone());
                            examples_open.set(false);
                        });

                        html! {
                            <div class="example-item" key={idx} onclick={load_example}>
                                <h4>{title}</h4>
                                <p>{desc}</p>
                            </div>
                        }
                    })}
                </div>
            </Modal>

            <Modal id="challenges" title="Challenges" active={*challenges_open} on_close={close_challenges}>
                {render_challenges_list(challenge_mode.clone(), current_challenge_id.clone(), program_code.clone(), challenges_open.clone())}
            </Modal>

            <Modal id="isaRef" title="ISA Reference" active={*isa_ref_open} on_close={close_isa_ref}>
                {html! { <div>{ISA_REF_CONTENT}</div> }}
            </Modal>

            <Modal id="help" title="Help" active={*help_open} on_close={close_help}>
                {html! { <div>{HELP_CONTENT}</div> }}
            </Modal>

            // GitHub Corner
            <a href="https://github.com/sw-embed/cor24-rs" class="github-corner" aria-label="View source on GitHub" target="_blank">
                <svg width="80" height="80" viewBox="0 0 250 250" style="fill:#00d9ff; color:#1a1a2e; position: absolute; top: 0; border: 0; right: 0;" aria-hidden="true">
                    <path d="M0,0 L115,115 L130,115 L142,142 L250,250 L250,0 Z"></path>
                    <path d="M128.3,109.0 C113.8,99.7 119.0,89.6 119.0,89.6 C122.0,82.7 120.5,78.6 120.5,78.6 C119.2,72.0 123.4,76.3 123.4,76.3 C127.3,80.9 125.5,87.3 125.5,87.3 C122.9,97.6 130.6,101.9 134.4,103.2" fill="currentColor" style="transform-origin: 130px 106px;" class="octo-arm"></path>
                    <path d="M115.0,115.0 C114.9,115.1 118.7,116.5 119.8,115.4 L133.7,101.6 C136.9,99.2 139.9,98.4 142.2,98.6 C133.8,88.0 127.5,74.4 143.8,58.0 C148.5,53.4 154.0,51.2 159.7,51.0 C160.3,49.4 163.2,43.6 171.4,40.1 C171.4,40.1 176.1,42.5 178.8,56.2 C183.1,58.6 187.2,61.8 190.9,65.4 C194.5,69.0 197.7,73.2 200.1,77.6 C213.8,80.2 216.3,84.9 216.3,84.9 C212.7,93.1 206.9,96.0 205.4,96.6 C205.1,102.4 203.0,107.8 198.3,112.5 C181.9,128.9 168.3,122.5 157.7,114.1 C157.9,116.9 156.7,120.9 152.7,124.9 L141.0,136.5 C139.8,137.7 141.6,141.9 141.8,141.8 Z" fill="currentColor" class="octo-body"></path>
                </svg>
            </a>

            // Footer
            <footer class="app-footer">
                <div class="footer-left">
                    <span>{"MIT License"}</span>
                    <span>{"© 2026 Michael A Wright"}</span>
                </div>
                <div class="footer-right">
                    <span>{format!("{} | {} | {}", env!("VERGEN_BUILD_HOST"), env!("VERGEN_GIT_SHA_SHORT"), env!("VERGEN_BUILD_TIMESTAMP"))}</span>
                </div>
            </footer>
        </div>
    }
}

// Helper function to render challenges list
fn render_challenges_list(
    challenge_mode: UseStateHandle<bool>,
    current_challenge_id: UseStateHandle<Option<usize>>,
    program_code: UseStateHandle<String>,
    challenges_open: UseStateHandle<bool>,
) -> Html {
    let challenges = get_challenges();

    html! {
        <div class="challenges-list">
            <h3>{"Available Challenges"}</h3>
            {for challenges.iter().map(|challenge| {
                let id = challenge.id;
                let name = challenge.name.clone();
                let description = challenge.description.clone();
                let hint = challenge.hint.clone();
                let initial_code = challenge.initial_code.clone();

                let challenge_mode = challenge_mode.clone();
                let current_challenge_id = current_challenge_id.clone();
                let program_code = program_code.clone();
                let challenges_open = challenges_open.clone();

                html! {
                    <div class="challenge-item">
                        <button
                            class="load-challenge-btn"
                            onclick={
                                let challenge_mode = challenge_mode.clone();
                                let current_challenge_id = current_challenge_id.clone();
                                let program_code = program_code.clone();
                                let challenges_open = challenges_open.clone();
                                let initial_code = initial_code.clone();
                                Callback::from(move |_| {
                                    challenge_mode.set(true);
                                    current_challenge_id.set(Some(id));
                                    program_code.set(initial_code.clone());
                                    challenges_open.set(false);
                                })
                            }
                        >
                            {format!("Load Challenge {}", id)}
                        </button>
                        <p><strong>{name}</strong></p>
                        <p>{description}</p>
                        <p><em>{"Hint: "}{hint}</em></p>
                    </div>
                }
            })}
        </div>
    }
}

// Constants for content
const EXAMPLE_PROGRAM: &str = "; COR24 Example: Basic Arithmetic
; Load constants and add them

        lc      r0,10       ; r0 = 10
        lc      r1,20       ; r1 = 20
        add     r0,r1       ; r0 = r0 + r1 = 30

        lc      r2,5        ; r2 = 5
        add     r0,r2       ; r0 = 35

        halt                ; Stop execution";

const TUTORIAL_CONTENT: &str = r#"
<h3>Welcome to the COR24 Assembly Emulator!</h3>
<p>This emulator teaches you assembly programming using the COR24 C-Oriented RISC architecture.</p>

<h4>CPU Features:</h4>
<ul>
    <li><strong>8 Registers (24-bit)</strong>: r0-r7 with special aliases</li>
    <li><strong>64KB Memory</strong>: Byte-addressable, little-endian</li>
    <li><strong>Single Condition Flag (C)</strong>: Set by compare instructions</li>
    <li><strong>Variable-length Instructions</strong>: 1-4 bytes</li>
</ul>

<h4>Register Aliases:</h4>
<ul>
    <li><code>fp (r3)</code> - Frame Pointer</li>
    <li><code>sp (r4)</code> - Stack Pointer</li>
    <li><code>z (r5)</code> - Zero Register (always 0)</li>
    <li><code>iv (r6)</code> - Interrupt Vector</li>
    <li><code>ir (r7)</code> - Interrupt Return</li>
</ul>

<h4>Basic Instructions:</h4>
<ul>
    <li><code>lc ra,dd</code> - Load constant (signed 8-bit)</li>
    <li><code>la ra,addr</code> - Load address (24-bit)</li>
    <li><code>add ra,rb</code> - Add registers</li>
    <li><code>add ra,dd</code> - Add immediate</li>
    <li><code>sub ra,rb</code> - Subtract registers</li>
    <li><code>cls ra,rb</code> - Compare less (signed), set C</li>
    <li><code>brt dd</code> - Branch if C=true</li>
    <li><code>brf dd</code> - Branch if C=false</li>
    <li><code>push ra</code> - Push to stack</li>
    <li><code>pop ra</code> - Pop from stack</li>
    <li><code>halt</code> - Stop execution</li>
</ul>
"#;

const ISA_REF_CONTENT: &str = r#"
<h3>COR24 Instruction Set Reference</h3>

<h4>Load Instructions</h4>
<p><strong>lc ra,dd</strong> - Load Constant (signed 8-bit)</p>
<p>Example: <code>lc r0,42</code> loads 42 into r0</p>

<p><strong>lcu ra,dd</strong> - Load Constant Unsigned</p>
<p>Example: <code>lcu r0,255</code> loads 255 into r0</p>

<p><strong>la ra,addr</strong> - Load 24-bit Address</p>
<p>Example: <code>la r0,0x1000</code> loads address into r0</p>

<h4>Arithmetic Instructions</h4>
<p><strong>add ra,rb</strong> - Add registers: ra = ra + rb</p>
<p><strong>add ra,dd</strong> - Add immediate: ra = ra + dd</p>
<p><strong>sub ra,rb</strong> - Subtract: ra = ra - rb</p>
<p><strong>mul ra,rb</strong> - Multiply: ra = ra * rb</p>

<h4>Logic Instructions</h4>
<p><strong>and ra,rb</strong> - Bitwise AND</p>
<p><strong>or ra,rb</strong> - Bitwise OR</p>
<p><strong>xor ra,rb</strong> - Bitwise XOR</p>
<p><strong>shl ra,rb</strong> - Shift left</p>
<p><strong>srl ra,rb</strong> - Shift right logical</p>
<p><strong>sra ra,rb</strong> - Shift right arithmetic</p>

<h4>Compare Instructions (set C flag)</h4>
<p><strong>ceq ra,rb</strong> - C = (ra == rb)</p>
<p><strong>cls ra,rb</strong> - C = (ra < rb) signed</p>
<p><strong>clu ra,rb</strong> - C = (ra < rb) unsigned</p>

<h4>Branch Instructions</h4>
<p><strong>bra dd</strong> - Branch always (PC-relative)</p>
<p><strong>brt dd</strong> - Branch if C=true</p>
<p><strong>brf dd</strong> - Branch if C=false</p>

<h4>Memory Instructions</h4>
<p><strong>lb ra,dd(rb)</strong> - Load byte signed</p>
<p><strong>lbu ra,dd(rb)</strong> - Load byte unsigned</p>
<p><strong>lw ra,dd(rb)</strong> - Load word (3 bytes)</p>
<p><strong>sb ra,dd(rb)</strong> - Store byte</p>
<p><strong>sw ra,dd(rb)</strong> - Store word</p>

<h4>Stack Instructions</h4>
<p><strong>push ra</strong> - Decrement sp, store ra</p>
<p><strong>pop ra</strong> - Load ra, increment sp</p>

<h4>Jump Instructions</h4>
<p><strong>jmp (ra)</strong> - Jump to address in ra</p>
<p><strong>jal ra,(rb)</strong> - Jump and link (call)</p>

<h4>Special</h4>
<p><strong>halt</strong> - Stop execution (la ir,addr form)</p>
<p><strong>mov ra,rb</strong> - Copy register</p>
<p><strong>mov ra,c</strong> - Move condition flag to register</p>
"#;

const HELP_CONTENT: &str = r#"
<h3>Help & Tips</h3>

<h4>How to Use:</h4>
<ol>
    <li><strong>Write Code</strong>: Enter your assembly program in the editor</li>
    <li><strong>Assemble</strong>: Click "Assemble" to parse and load your program</li>
    <li><strong>Step/Run</strong>: Use "Step" to execute one instruction or "Run" to complete</li>
    <li><strong>Reset</strong>: Click "Reset" to clear the CPU and start over</li>
</ol>

<h4>Assembly Syntax:</h4>
<ul>
    <li>Labels end with colon: <code>loop:</code></li>
    <li>Comments start with semicolon: <code>; comment</code></li>
    <li>Numbers: decimal (42), hex (0x2A)</li>
    <li>Register names: r0-r7, fp, sp, z, iv, ir</li>
</ul>

<h4>Calling Convention:</h4>
<pre>
; Function prologue
push    fp          ; Save frame pointer
push    r2          ; Save callee-saved
push    r1          ; Save return address
mov     fp,sp       ; Set up frame

; Function body...

; Function epilogue
mov     sp,fp       ; Restore stack
pop     r1          ; Restore r1
pop     r2          ; Restore r2
pop     fp          ; Restore fp
jmp     (r1)        ; Return
</pre>

<h4>Debugging Tips:</h4>
<ul>
    <li>Use "Step" to execute one instruction at a time</li>
    <li>Watch registers to see value changes (highlighted)</li>
    <li>Check the memory viewer to see your program</li>
    <li>The condition flag C is shown in the legend</li>
</ul>
"#;
