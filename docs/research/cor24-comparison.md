# COR24-TB Development Board Comparison

## Introduction

This document provides a comprehensive comparison of the COR24-TB FPGA development board against popular microcontroller and single-board computer platforms. The COR24-TB features a custom 24-bit soft CPU implemented on a Lattice MachXO2 FPGA, representing a unique position in the embedded systems landscape as an open-source, educational, and hackable processor design.

The comparison covers various metrics including processing power, memory capacity, I/O capabilities, connectivity, and intended use cases. This analysis helps developers and educators understand where the COR24-TB fits relative to mainstream development platforms.

---

## Overview of COR24-TB

**Source:** [MakerLisp COR24-TB Product Page](https://www.makerlisp.com/cor24-test-board)

The COR24-TB is a soft CPU development board featuring:

| Specification | Value |
|---------------|-------|
| **FPGA** | Lattice LCMXO2280C-5TN144C (MachXO2) |
| **CPU Architecture** | COR24 - Custom 24-bit RISC |
| **Clock Speed** | 101.6 MHz (3x 33.8688 MHz via PLL) |
| **SRAM** | 1 MB (ISSI IS61WV10248EDBLL-10TLI) |
| **On-chip RAM** | 4 KB embedded block RAM |
| **Register Width** | 24-bit |
| **Instruction Set** | 32 instructions, variable-length encoding |
| **GPIO** | 10 general purpose I/O pins |
| **UART** | 921600 baud via external USB bridge |
| **User I/O** | 1 LED, 1 pushbutton |
| **Power** | USB 5V, 100mA available for external circuits |
| **Form Factor** | Compact PCB with JTAG header |
| **License** | Open-source (MIT) |

**Unique Characteristics:**
- Fully open-source Verilog CPU design
- Educational platform for understanding CPU architecture
- FPGA-based - can be modified, extended, or replaced entirely
- Supports development with Lattice Diamond toolchain

---

## Comparison Tables

### Table 1: Processing Power Comparison

| Platform | Architecture | Bits | Cores | Clock Speed | Est. MIPS | Notes |
|----------|-------------|------|-------|-------------|-----------|-------|
| **COR24-TB** | Custom RISC | 24 | 1 | 101.6 MHz | ~25-50* | Soft CPU on FPGA |
| Arduino Uno R3 | AVR (ATmega328P) | 8 | 1 | 16 MHz | ~16 | Classic baseline |
| Arduino Uno R4 | ARM Cortex-M4 | 32 | 1 | 48 MHz | ~50 | 3x faster than R3 |
| Arduino Nano ESP32 | Xtensa LX7 | 32 | 2 | 240 MHz | ~400+ | Dual-core powerhouse |
| ESP32-C3 | RISC-V | 32 | 1 | 160 MHz | ~160 | Single-core RISC-V |
| ESP32-C6 | RISC-V | 32 | 1 | 160 MHz | ~160 | WiFi 6 + Thread |
| ESP32-H2 | RISC-V | 32 | 1 | 96 MHz | ~96 | Low-power Thread/Zigbee |
| ESP32-S3 | Xtensa LX7 | 32 | 2 | 240 MHz | ~400+ | AI/ML capable |
| STM32F103 (Blue Pill) | ARM Cortex-M3 | 32 | 1 | 72 MHz | ~90 | Popular hobbyist board |
| STM32F411 (Black Pill) | ARM Cortex-M4F | 32 | 1 | 100 MHz | ~125 | FPU + DSP |
| PIC32MX | MIPS M4K | 32 | 1 | 80 MHz | ~83 | Classic Microchip |
| PIC32MZ | MIPS M-Class | 32 | 1 | 200 MHz | ~330 | High-performance PIC |
| 8051 (Classic) | CISC | 8 | 1 | 12 MHz | ~1 | 12 clocks/instruction |
| 8051 (Modern) | CISC | 8 | 1 | 100 MHz | ~100 | 1 clock/instruction |
| Raspberry Pi Zero 2 W | ARM Cortex-A53 | 64 | 4 | 1 GHz | ~2000+ | Full Linux SBC |
| Raspberry Pi 4 | ARM Cortex-A72 | 64 | 4 | 1.5 GHz | ~5000+ | Desktop-class |
| Raspberry Pi 5 | ARM Cortex-A76 | 64 | 4 | 2.4 GHz | ~10000+ | Latest flagship |

*COR24 MIPS estimate based on instruction mix; actual performance varies by workload. Multi-cycle instructions (MUL: 24 cycles, shifts: 1 cycle/bit) reduce average throughput.

### Table 2: Memory Comparison

| Platform | RAM | Flash/ROM | Address Space | Memory Type |
|----------|-----|-----------|---------------|-------------|
| **COR24-TB** | 1 MB SRAM + 4 KB EBR | External load | 24-bit (16 MB) | Static RAM |
| Arduino Uno R3 | 2 KB | 32 KB | 16-bit | SRAM + Flash |
| Arduino Uno R4 | 32 KB | 256 KB | 32-bit | SRAM + Flash |
| Arduino Nano ESP32 | 512 KB + 8 MB PSRAM | 16 MB | 32-bit | SRAM + PSRAM + Flash |
| ESP32-C3 | 400 KB | 4 MB | 32-bit | SRAM + Flash |
| ESP32-C6 | 512 KB | 4 MB | 32-bit | SRAM + Flash |
| ESP32-H2 | 320 KB | 4 MB | 32-bit | SRAM + Flash |
| ESP32-S3 | 512 KB + 8 MB PSRAM | 8-16 MB | 32-bit | SRAM + PSRAM + Flash |
| STM32F103 | 20 KB | 64-128 KB | 32-bit | SRAM + Flash |
| STM32F411 | 128 KB | 512 KB | 32-bit | SRAM + Flash |
| PIC32MX | 16-128 KB | 64-512 KB | 32-bit | SRAM + Flash |
| PIC32MZ | 512 KB | 2 MB | 32-bit | SRAM + Flash |
| 8051 (Classic) | 128 B internal | 4 KB | 16-bit | RAM + ROM |
| 8051 (Modern) | 256 B + XRAM | 64 KB | 16-bit | RAM + Flash |
| Pi Zero 2 W | 512 MB | SD Card | 32/64-bit | LPDDR2 |
| Raspberry Pi 4 | 2-8 GB | SD Card | 64-bit | LPDDR4 |
| Raspberry Pi 5 | 4-16 GB | SD Card | 64-bit | LPDDR4X |

### Table 3: Connectivity & I/O Comparison

| Platform | WiFi | Bluetooth | GPIO | UART | I2C | SPI | USB | Other |
|----------|------|-----------|------|------|-----|-----|-----|-------|
| **COR24-TB** | - | - | 10 | 1 (921k) | Bit-bang* | Bit-bang* | Power only | JTAG |
| Arduino Uno R3 | - | - | 14 | 1 | 1 | 1 | CDC | - |
| Arduino Uno R4 | - | - | 14 | 1 | 1 | 1 | USB-C | DAC, CAN |
| Arduino Nano ESP32 | 802.11 b/g/n | BLE 5.0 | 14 | 2 | 2 | 2 | USB-C | - |
| ESP32-C3 | 802.11 b/g/n | BLE 5.0 | 19 | 2 | 1 | 3 | USB | - |
| ESP32-C6 | 802.11ax (WiFi 6) | BLE 5.3 | 19 | 2 | 1 | 1 | USB | Thread, Zigbee |
| ESP32-H2 | - | BLE 5.0 | 19 | 2 | 1 | 2 | USB | Thread, Zigbee |
| ESP32-S3 | 802.11 b/g/n | BLE 5.0 | 45 | 3 | 2 | 4 | USB OTG | Camera, LCD |
| STM32F103 | - | - | 37 | 3 | 2 | 2 | USB | CAN |
| STM32F411 | - | - | 50 | 3 | 3 | 5 | USB OTG | I2S |
| PIC32MX | - | - | Varies | 2-6 | 2 | 2-4 | USB | CAN |
| 8051 | - | - | 32 | 1 | - | - | - | Timers |
| Pi Zero 2 W | 802.11 b/g/n | BLE 4.2 | 40 | 2 | 1 | 2 | Micro USB | CSI, HAT |
| Raspberry Pi 4 | 802.11ac | BLE 5.0 | 40 | 6 | 6 | 7 | 4x USB | PCIe, HDMI |
| Raspberry Pi 5 | 802.11ac | BLE 5.0 | 40 | 6 | 6 | 7 | 4x USB 3 | PCIe x4, HDMI |

*COR24-TB I2C/SPI support via GPIO bit-banging (in development)

### Table 4: Power & Physical Characteristics

| Platform | Operating Voltage | Typical Current | Deep Sleep | Form Factor | Price (USD) |
|----------|-------------------|-----------------|------------|-------------|-------------|
| **COR24-TB** | 5V (USB) | ~50-100 mA | N/A | Custom PCB | ~$190 |
| Arduino Uno R3 | 5V | ~50 mA | ~15 mA | 68.6 x 53.4 mm | ~$27 |
| Arduino Uno R4 | 5V | ~100 mA | ~100 µA | 68.6 x 53.4 mm | ~$27 |
| Arduino Nano ESP32 | 3.3V | ~100 mA | ~7 µA | 45 x 18 mm | ~$20 |
| ESP32-C3 | 3.3V | ~80 mA | ~5 µA | Module | ~$3-5 |
| ESP32-C6 | 3.3V | ~90 mA | ~7 µA | Module | ~$4-6 |
| ESP32-H2 | 3.3V | ~70 mA | ~5 µA | Module | ~$3-5 |
| ESP32-S3 | 3.3V | ~100 mA | ~7 µA | Module | ~$5-8 |
| Seeed XIAO ESP32C3 | 3.3V | ~80 mA | ~14 µA | 21 x 17.8 mm | ~$5-9 |
| Seeed XIAO ESP32S3 | 3.3V | ~100 mA | ~14 µA | 21 x 17.8 mm | ~$7-14 |
| STM32F103 Blue Pill | 3.3V | ~25 mA | ~2 µA | 53 x 23 mm | ~$3-5 |
| PIC32MX | 3.3V | ~30 mA | ~0.5 µA | DIP/QFP | ~$5-15 |
| 8051 | 5V/3.3V | ~20 mA | ~1 µA | DIP-40 | ~$1-3 |
| Pi Zero 2 W | 5V | ~300 mA | N/A | 65 x 30 mm | ~$15 |
| Raspberry Pi 4 | 5V | ~600-1200 mA | N/A | 85 x 56 mm | ~$35-75 |
| Raspberry Pi 5 | 5V | ~800-2000 mA | N/A | 85 x 56 mm | ~$60-100 |

### Table 5: Software & Development Ecosystem

| Platform | Primary Language | IDE/Toolchain | RTOS Support | Linux | Open Source HW |
|----------|-----------------|---------------|--------------|-------|----------------|
| **COR24-TB** | C, Assembly | Lattice Diamond, GCC | Custom | No | **Yes (MIT)** |
| Arduino Uno R3/R4 | C/C++ | Arduino IDE | FreeRTOS | No | Partial |
| ESP32 Family | C/C++, Python | Arduino, ESP-IDF | FreeRTOS | No | No |
| STM32 | C/C++ | STM32CubeIDE, Keil | FreeRTOS, Zephyr | No | No |
| PIC32 | C | MPLAB X | FreeRTOS | No | No |
| 8051 | C, Assembly | Keil, SDCC | Custom | No | No |
| Raspberry Pi | Python, C, etc. | Any | Yes | **Yes** | Partial |

---

## Observations

### Strengths of COR24-TB

1. **Fully Open Architecture**: Unlike all other platforms compared, the COR24-TB's CPU is completely open-source Verilog. This allows:
   - Study of actual CPU implementation at the RTL level
   - Modification of the instruction set
   - Addition of custom hardware accelerators
   - Educational use for computer architecture courses

2. **Generous RAM**: With 1 MB of SRAM, the COR24-TB has more RAM than most microcontrollers:
   - More than Arduino Uno R4 (32 KB)
   - More than most ESP32 variants (320-512 KB)
   - Comparable to high-end PIC32MZ

3. **Reasonable Clock Speed**: At 101.6 MHz, performance is competitive with mid-range MCUs.

4. **Simple, Clean Architecture**: The 24-bit, 32-instruction RISC design is easier to understand and emulate than complex modern ISAs.

5. **FPGA Flexibility**: The underlying FPGA can be reprogrammed for entirely different applications.

### Limitations of COR24-TB

1. **Premium Pricing**: At ~$190, it's more expensive than a Raspberry Pi 5 (16GB: ~$100) and significantly more than ESP32 boards ($3-15). The price reflects low-volume, hand-assembled production and the educational/niche market positioning.

2. **No Wireless Connectivity**: Unlike ESP32 family, Arduino Nano ESP32, or Raspberry Pi, there's no built-in WiFi or Bluetooth.

3. **Limited Peripheral Support**: Current implementation lacks hardware I2C, SPI, ADC, DAC (bit-banging possible for I2C/SPI).

4. **Non-Standard Word Size**: 24-bit architecture is unusual; most compilers and tools target 8, 16, 32, or 64-bit.

5. **Smaller Ecosystem**: Limited library and community support compared to Arduino or ESP32.

6. **FPGA Constraints**: The MachXO2280C has limited logic resources; major CPU extensions may not fit.

7. **Development Complexity**: Requires Lattice Diamond for HDL modifications; no plug-and-play Arduino experience.

### Positioning in the Market

| Category | Best Choice | Where COR24-TB Fits |
|----------|------------|---------------------|
| **Education - CPU Architecture** | **COR24-TB** | Ideal - open RTL, simple ISA (if budget allows) |
| **Education - Programming** | Arduino | Secondary - steeper learning curve, higher cost |
| **IoT / Wireless Projects** | ESP32 Family | Not suitable - no wireless |
| **Cost-Sensitive Production** | ESP32-C3, 8051 | Not suitable - $190 vs $3-5 |
| **Maximum Performance** | Raspberry Pi 5 | Not competitive (Pi 5 is faster AND cheaper) |
| **Ultra-Low Power** | ESP32-H2, PIC32MM | Not suitable - FPGA power overhead |
| **Custom Hardware Exploration** | **COR24-TB** | Ideal - reprogrammable FPGA (premium price) |
| **Retro Computing / Emulation** | **COR24-TB** | Good fit - simple architecture |
| **Budget-Conscious Hobbyist** | ESP32, STM32 | Not suitable - 10-50x more expensive |

### Comparable Historical Systems

The COR24 architecture is reminiscent of classic 1970s-80s minicomputers:

| System | Word Size | Registers | Clock | Era |
|--------|-----------|-----------|-------|-----|
| COR24 | 24-bit | 8 | 101 MHz | 2024 |
| PDP-8 | 12-bit | 1 (AC) | 1.5 MHz | 1965 |
| PDP-11 | 16-bit | 8 | 15 MHz | 1970 |
| Motorola 68000 | 32-bit | 16 | 8 MHz | 1979 |
| ARM2 | 32-bit | 16 | 8 MHz | 1986 |

The COR24's clean RISC design with 8 general-purpose registers is philosophically similar to early RISC processors like the Berkeley RISC-I (1982) or ARM1 (1985), but with modern FPGA implementation.

---

## Summary

The **COR24-TB** occupies a unique niche in the embedded development landscape:

**Primary Use Cases:**
- Computer architecture education
- CPU design exploration and experimentation
- Hobbyist FPGA development
- Retro computing projects
- Building custom emulators and studying ISA design

**Not Ideal For:**
- Production IoT devices (use ESP32)
- High-performance computing (use Raspberry Pi)
- Battery-powered projects (use low-power MCUs)
- Beginners seeking plug-and-play experience (use Arduino)

**Key Differentiator:** The COR24-TB is the only platform in this comparison where you can read, understand, and modify the actual CPU implementation. This makes it invaluable for education and experimentation, even though it may not be the optimal choice for production applications.

For developers interested in understanding how processors work at a fundamental level, or for those wanting a platform to experiment with custom instruction sets and hardware, the COR24-TB offers capabilities that no commercial microcontroller can match.

---

## Sources

- [MakerLisp COR24-TB Product Page](https://www.makerlisp.com/cor24-test-board)
- [Arduino UNO R4 Minima Documentation](https://docs.arduino.cc/hardware/uno-r4-minima)
- [Raspberry Pi 5 Specifications](https://www.raspberrypi.com/products/raspberry-pi-5/)
- [Raspberry Pi 4 Specifications](https://www.raspberrypi.com/products/raspberry-pi-4-model-b/specifications/)
- [Raspberry Pi Zero 2 W](https://www.raspberrypi.com/products/raspberry-pi-zero-2-w/)
- [ESP32 Comparison Guide](https://www.espboards.dev/blog/esp32-soc-options/)
- [ESP32-C3/C6/H2 Comparison](https://openelab.io/blogs/learn/key-differences-betweenesp32-c3-esp32-c6-and-esp32-h2)
- [Seeed XIAO ESP32 Comparison](https://www.seeedstudio.com/blog/2026/01/14/xiao-esp32-s3-c3-c6-comparison/)
- [ESP32-S3 Datasheet](https://www.espressif.com/sites/default/files/documentation/esp32-s3_datasheet_en.pdf)
- [Arduino Nano ESP32 Specifications](https://store.arduino.cc/products/nano-esp32)
- [STM32F4 Series Overview](https://www.st.com/en/microcontrollers-microprocessors/stm32f4-series.html)
- [STM32 Blue Pill vs Black Pill](https://hackaday.com/2021/01/20/blue-pill-vs-black-pill-transitioning-from-stm32f103-to-stm32f411/)
- [PIC32 Architecture Overview](https://www.microchip.com/en-us/products/microcontrollers/32-bit-mcus/pic32m)
- [8051 Microcontroller Architecture](https://www.geeksforgeeks.org/electronics-engineering/8051-microcontroller-architecture/)
- [Intel MCS-51 (8051) Wikipedia](https://en.wikipedia.org/wiki/Intel_MCS-51)
