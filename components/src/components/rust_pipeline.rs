//! Rust Pipeline view component
//! Shows the compilation pipeline: Rust -> WASM -> COR24 Assembly -> Machine Code -> Execution

use yew::prelude::*;
use super::collapsible::Collapsible;

/// Pre-built example for the Rust pipeline demo
#[derive(Clone, PartialEq)]
pub struct RustExample {
    pub name: String,
    pub description: String,
    pub rust_source: String,
    pub wasm_hex: String,
    pub wasm_size: usize,
    pub wasm_disassembly: String,
    pub cor24_assembly: String,
    pub machine_code_hex: String,
    pub machine_code_size: usize,
    pub listing: String,
}

/// CPU state for display in the Rust pipeline execution panel
#[derive(Clone, PartialEq, Default)]
pub struct RustCpuState {
    pub registers: [u32; 8],
    pub pc: u32,
    pub condition_flag: bool,
    pub is_halted: bool,
    pub led_value: u8,
    pub cycle_count: u32,
    pub memory_snapshot: Vec<u8>,
    pub current_instruction: String,
}

#[derive(Properties, PartialEq)]
pub struct RustPipelineProps {
    pub examples: Vec<RustExample>,
    pub on_load: Callback<RustExample>,
    pub on_step: Callback<()>,
    pub on_run: Callback<()>,
    pub on_reset: Callback<()>,
    pub cpu_state: RustCpuState,
    pub is_loaded: bool,
    pub is_running: bool,
}

#[function_component(RustPipeline)]
pub fn rust_pipeline(props: &RustPipelineProps) -> Html {
    let selected_example = use_state(|| {
        props.examples.first().cloned()
    });

    let on_example_select = {
        let selected_example = selected_example.clone();
        let examples = props.examples.clone();
        Callback::from(move |e: Event| {
            let target = e.target_dyn_into::<web_sys::HtmlSelectElement>();
            if let Some(select) = target {
                let idx = select.selected_index() as usize;
                if let Some(example) = examples.get(idx) {
                    selected_example.set(Some(example.clone()));
                }
            }
        })
    };

    let on_load_click = {
        let on_load = props.on_load.clone();
        let selected = selected_example.clone();
        Callback::from(move |_| {
            if let Some(example) = &*selected {
                on_load.emit(example.clone());
            }
        })
    };

    let on_step_click = {
        let on_step = props.on_step.clone();
        Callback::from(move |_| on_step.emit(()))
    };

    let on_run_click = {
        let on_run = props.on_run.clone();
        Callback::from(move |_| on_run.emit(()))
    };

    let on_reset_click = {
        let on_reset = props.on_reset.clone();
        Callback::from(move |_| on_reset.emit(()))
    };

    let state = &props.cpu_state;

    html! {
        <div class="rust-pipeline">
            // Example selector
            <div class="pipeline-header">
                <label>{"Example: "}</label>
                <select onchange={on_example_select} disabled={props.is_running}>
                    {for props.examples.iter().map(|ex| {
                        html! {
                            <option value={ex.name.clone()}>{&ex.name}{" - "}{&ex.description}</option>
                        }
                    })}
                </select>
                <button class="load-btn" onclick={on_load_click} disabled={props.is_running}>
                    {"Load"}
                </button>
            </div>

            if let Some(example) = &*selected_example {
                // Rust Source
                <div class="pipeline-stage">
                    <h3>{"1. Rust Source"}</h3>
                    <pre class="code-block rust-code">{&example.rust_source}</pre>
                </div>

                // WASM Binary (collapsible)
                <Collapsible title="2. WASM Binary" badge={Some(format!("{} bytes", example.wasm_size))}>
                    <pre class="code-block hex-dump">{&example.wasm_hex}</pre>
                </Collapsible>

                // WASM Disassembly (collapsible)
                <Collapsible title="2b. WASM Disassembly">
                    <pre class="code-block wasm-disasm">{&example.wasm_disassembly}</pre>
                </Collapsible>

                // COR24 Assembly (collapsible, initially open)
                <Collapsible title="3. COR24 Assembly" initially_open={true}>
                    <pre class="code-block asm-code">{&example.cor24_assembly}</pre>
                </Collapsible>

                // Machine Code (collapsible)
                <Collapsible title="4. Machine Code" badge={Some(format!("{} bytes", example.machine_code_size))}>
                    <pre class="code-block hex-dump">{&example.machine_code_hex}</pre>
                    <h4>{"Listing:"}</h4>
                    <pre class="code-block listing">{&example.listing}</pre>
                </Collapsible>

                // Execution panel with controls and state
                <div class="pipeline-stage execution-panel">
                    <h3>{"5. Execution"}</h3>

                    // Control buttons
                    <div class="execution-controls">
                        <button class="step-btn" onclick={on_step_click}
                            disabled={!props.is_loaded || state.is_halted || props.is_running}>
                            {"Step"}
                        </button>
                        <button class="run-btn" onclick={on_run_click}
                            disabled={!props.is_loaded || state.is_halted || props.is_running}>
                            {if props.is_running { "Running..." } else { "Run" }}
                        </button>
                        <button class="reset-btn" onclick={on_reset_click}
                            disabled={!props.is_loaded || props.is_running}>
                            {"Reset"}
                        </button>
                        if state.is_halted {
                            <span class="status-halted">{"HALTED"}</span>
                        }
                    </div>

                    // Current instruction
                    if props.is_loaded && !state.current_instruction.is_empty() {
                        <div class="current-instruction">
                            <span class="label">{"Next: "}</span>
                            <code>{&state.current_instruction}</code>
                        </div>
                    }

                    // Two-column layout for registers and I/O
                    <div class="execution-state">
                        // Left column: Registers
                        <div class="registers-panel">
                            <h4>{"Registers"}</h4>
                            <div class="register-grid">
                                {for (0..8).map(|i| {
                                    let name = match i {
                                        0 => "r0",
                                        1 => "r1",
                                        2 => "r2",
                                        3 => "fp",
                                        4 => "sp",
                                        5 => "z",
                                        6 => "iv",
                                        7 => "ir",
                                        _ => "??",
                                    };
                                    let val = state.registers[i];
                                    html! {
                                        <div class="register-row">
                                            <span class="reg-name">{name}</span>
                                            <span class="reg-value">{format!("0x{:06X}", val)}</span>
                                        </div>
                                    }
                                })}
                                <div class="register-row">
                                    <span class="reg-name">{"PC"}</span>
                                    <span class="reg-value">{format!("0x{:06X}", state.pc)}</span>
                                </div>
                                <div class="register-row">
                                    <span class="reg-name">{"C"}</span>
                                    <span class="reg-value">{if state.condition_flag { "1" } else { "0" }}</span>
                                </div>
                            </div>
                        </div>

                        // Right column: I/O
                        <div class="io-panel">
                            <h4>{"I/O Peripherals"}</h4>

                            // LEDs
                            <div class="led-display">
                                <span class="led-label">{"LEDs: "}</span>
                                <div class="led-row">
                                    {for (0..8).rev().map(|i| {
                                        let led_on = (state.led_value >> i) & 1 == 1;
                                        let class = if led_on { "led led-on" } else { "led led-off" };
                                        html! {
                                            <div class={class}>{i}</div>
                                        }
                                    })}
                                </div>
                                <span class="led-value">{format!("0x{:02X}", state.led_value)}</span>
                            </div>

                            // Cycle count
                            <div class="cycle-count">
                                <span>{"Cycles: "}{state.cycle_count}</span>
                            </div>

                            // I2C placeholder
                            <div class="i2c-panel">
                                <span class="i2c-label">{"I2C Bus: "}</span>
                                <span class="i2c-status">{"(not connected)"}</span>
                            </div>
                        </div>
                    </div>

                    // Memory viewer (first 64 bytes of stack area)
                    if props.is_loaded && !state.memory_snapshot.is_empty() {
                        <Collapsible title="Memory (Stack)" badge={Some(format!("{} bytes", state.memory_snapshot.len()))}>
                            <pre class="code-block memory-dump">{
                                format_memory_dump(&state.memory_snapshot, 0xFFFFC0)
                            }</pre>
                        </Collapsible>
                    }
                </div>
            }

            // Future: Server-side compilation notice
            <div class="pipeline-note">
                <em>{"Note: Examples are pre-built. Server-side compilation coming soon."}</em>
            </div>
        </div>
    }
}

/// Format memory as hex dump with addresses
fn format_memory_dump(data: &[u8], base_addr: u32) -> String {
    let mut output = String::new();
    for (i, chunk) in data.chunks(16).enumerate() {
        let addr = base_addr + (i * 16) as u32;
        output.push_str(&format!("{:06X}: ", addr));
        for (j, byte) in chunk.iter().enumerate() {
            output.push_str(&format!("{:02X} ", byte));
            if j == 7 {
                output.push(' ');
            }
        }
        output.push('\n');
    }
    output
}
