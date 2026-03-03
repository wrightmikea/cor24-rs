# COR24-TB Development Board Comparison (Visual Edition)

## Introduction

This document provides visual comparisons of the COR24-TB FPGA development board against popular microcontroller and single-board computer platforms using Mermaid diagrams and formatted tables.

---

## COR24-TB Quick Specs

```mermaid
mindmap
  root((COR24-TB))
    CPU
      24-bit RISC
      101.6 MHz
      8 Registers
      32 Instructions
    Memory
      1 MB SRAM
      4 KB EBR
      16 MB Address Space
    I/O
      10 GPIO
      921600 UART
      1 LED
      1 Button
    Platform
      Lattice MachXO2
      Open Source
      MIT License
      JTAG Programming
```

---

## Clock Speed Comparison

```mermaid
xychart-beta
    title "Clock Speed Comparison (MHz)"
    x-axis ["COR24", "Uno R3", "Uno R4", "ESP32-C3", "ESP32-S3", "STM32F1", "PIC32MX", "Pi Zero2", "Pi 4", "Pi 5"]
    y-axis "MHz" 0 --> 2500
    bar [101.6, 16, 48, 160, 240, 72, 80, 1000, 1500, 2400]
```

---

## Performance Tiers

```mermaid
quadrantChart
    title Performance vs Openness
    x-axis Low Performance --> High Performance
    y-axis Closed Source --> Open Source
    quadrant-1 Open & Powerful
    quadrant-2 Open but Limited
    quadrant-3 Closed & Limited
    quadrant-4 Closed but Powerful

    COR24-TB: [0.15, 0.95]
    Arduino Uno R3: [0.05, 0.40]
    Arduino Uno R4: [0.12, 0.40]
    ESP32-C3: [0.25, 0.20]
    ESP32-S3: [0.45, 0.20]
    STM32F103: [0.18, 0.15]
    PIC32MX: [0.16, 0.10]
    Pi Zero 2: [0.55, 0.35]
    Pi 4: [0.75, 0.35]
    Pi 5: [0.90, 0.35]
```

---

## Architecture Family Tree

```mermaid
flowchart TB
    subgraph "8-bit Legacy"
        A8051[8051<br/>8-bit CISC<br/>12-100 MHz]
        AVR[AVR ATmega<br/>8-bit RISC<br/>16-20 MHz]
    end

    subgraph "24-bit Custom"
        COR24[COR24<br/>24-bit RISC<br/>101.6 MHz<br/>OPEN SOURCE]
        style COR24 fill:#90EE90,stroke:#006400,stroke-width:3px
    end

    subgraph "32-bit ARM"
        CM3[Cortex-M3<br/>STM32F103<br/>72 MHz]
        CM4[Cortex-M4<br/>Arduino R4<br/>48-100 MHz]
        CA53[Cortex-A53<br/>Pi Zero 2<br/>1 GHz]
        CA72[Cortex-A72<br/>Pi 4<br/>1.5 GHz]
        CA76[Cortex-A76<br/>Pi 5<br/>2.4 GHz]
    end

    subgraph "32-bit RISC-V"
        RV32[RISC-V 32<br/>ESP32-C3/C6/H2<br/>96-160 MHz]
    end

    subgraph "32-bit Xtensa"
        XT[Xtensa LX7<br/>ESP32-S3<br/>240 MHz]
    end

    subgraph "32-bit MIPS"
        MIPS[MIPS M4K<br/>PIC32<br/>80-200 MHz]
    end
```

---

## Memory Comparison (Log Scale)

```mermaid
xychart-beta
    title "RAM Size Comparison (KB, Log Scale Approximation)"
    x-axis ["8051", "Uno R3", "Uno R4", "ESP32-C3", "COR24", "ESP32-S3", "Pi Zero2", "Pi 4", "Pi 5"]
    y-axis "RAM (KB)" 0 --> 1200
    bar [0.128, 2, 32, 400, 1024, 512, 512, 1000, 1000]
```

*Note: Pi Zero 2 = 512MB, Pi 4 = 2-8GB, Pi 5 = 4-16GB (shown capped for scale)*

---

## Feature Matrix

```mermaid
block-beta
    columns 10

    block:header:10
        h1["Platform"] h2["WiFi"] h3["BLE"] h4["GPIO"] h5["USB"] h6["Linux"] h7["Open HW"] h8["Price"] h9["Power"]
    end

    block:cor24:10
        p1["COR24-TB"] w1["---"] b1["---"] g1["10"] u1["Pwr"] l1["---"] o1["YES"] pr1["$190"] pw1["Med"]
    end

    block:arduino:10
        p2["Uno R4"] w2["---"] b2["---"] g2["14"] u2["CDC"] l2["---"] o2["Part"] pr2["$27"] pw2["Med"]
    end

    block:esp32:10
        p3["ESP32-S3"] w3["YES"] b3["YES"] g3["45"] u3["OTG"] l3["---"] o3["---"] pr3["$8"] pw3["Med"]
    end

    block:stm32:10
        p4["STM32F1"] w4["---"] b4["---"] g4["37"] u4["USB"] l4["---"] o4["---"] pr4["$5"] pw4["Low"]
    end

    block:pi5:10
        p5["Pi 5"] w5["YES"] b5["YES"] g5["40"] u5["4x3"] l5["YES"] o5["Part"] pr5["$80"] pw5["High"]
    end

    style o1 fill:#90EE90
```

---

## Connectivity Comparison

```mermaid
pie showData
    title "GPIO Count Distribution"
    "COR24-TB (10)" : 10
    "Arduino Uno (14)" : 14
    "ESP32-C3 (19)" : 19
    "STM32F103 (37)" : 37
    "Raspberry Pi (40)" : 40
    "ESP32-S3 (45)" : 45
```

---

## Use Case Decision Tree

```mermaid
flowchart TD
    START([What do you need?]) --> Q1{Learn CPU<br/>Architecture?}

    Q1 -->|Yes| COR24[COR24-TB<br/>Open Verilog CPU]
    style COR24 fill:#90EE90,stroke:#006400,stroke-width:3px

    Q1 -->|No| Q2{Need<br/>WiFi/BLE?}

    Q2 -->|Yes| Q3{Need<br/>Linux?}
    Q2 -->|No| Q4{Budget<br/>Constrained?}

    Q3 -->|Yes| PI[Raspberry Pi<br/>4 or 5]
    Q3 -->|No| ESP[ESP32 Family<br/>C3/C6/S3]

    Q4 -->|Yes| Q5{8-bit OK?}
    Q4 -->|No| Q6{Need<br/>USB OTG?}

    Q5 -->|Yes| A8051[8051 or AVR]
    Q5 -->|No| STM[STM32 Blue Pill]

    Q6 -->|Yes| STM32[STM32F4<br/>Black Pill]
    Q6 -->|No| PIC[PIC32MX]

    START --> Q7{Custom<br/>Hardware?}
    Q7 -->|Yes| COR24
```

---

## Detailed Comparison Tables

### Processing Power

| Platform | Arch | Bits | Cores | MHz | MIPS | Category |
|:---------|:----:|:----:|:-----:|----:|-----:|:---------|
| **COR24-TB** | Custom RISC | 24 | 1 | 101.6 | ~40 | Soft CPU |
| Arduino Uno R3 | AVR | 8 | 1 | 16 | 16 | Classic |
| Arduino Uno R4 | ARM Cortex-M4 | 32 | 1 | 48 | 50 | Modern |
| Arduino Nano ESP32 | Xtensa LX7 | 32 | 2 | 240 | 400+ | WiFi |
| ESP32-C3 | RISC-V | 32 | 1 | 160 | 160 | WiFi |
| ESP32-C6 | RISC-V | 32 | 1 | 160 | 160 | WiFi 6 |
| ESP32-H2 | RISC-V | 32 | 1 | 96 | 96 | Thread |
| ESP32-S3 | Xtensa LX7 | 32 | 2 | 240 | 400+ | AI/ML |
| STM32F103 | ARM Cortex-M3 | 32 | 1 | 72 | 90 | Dev |
| STM32F411 | ARM Cortex-M4F | 32 | 1 | 100 | 125 | Dev |
| PIC32MX | MIPS M4K | 32 | 1 | 80 | 83 | Classic |
| PIC32MZ | MIPS M-Class | 32 | 1 | 200 | 330 | Hi-Perf |
| 8051 Classic | CISC | 8 | 1 | 12 | 1 | Legacy |
| Pi Zero 2 W | ARM Cortex-A53 | 64 | 4 | 1000 | 2000+ | Linux |
| Raspberry Pi 4 | ARM Cortex-A72 | 64 | 4 | 1500 | 5000+ | Linux |
| Raspberry Pi 5 | ARM Cortex-A76 | 64 | 4 | 2400 | 10000+ | Linux |

### Memory

| Platform | RAM | Flash | Address Space |
|:---------|----:|------:|:--------------|
| **COR24-TB** | 1 MB + 4 KB | External | 16 MB (24-bit) |
| Arduino Uno R3 | 2 KB | 32 KB | 64 KB |
| Arduino Uno R4 | 32 KB | 256 KB | 4 GB |
| ESP32-C3 | 400 KB | 4 MB | 4 GB |
| ESP32-S3 | 512 KB + 8 MB | 16 MB | 4 GB |
| STM32F103 | 20 KB | 64 KB | 4 GB |
| PIC32MX | 128 KB | 512 KB | 4 GB |
| 8051 | 128 B | 4 KB | 64 KB |
| Pi Zero 2 W | 512 MB | SD | 4+ GB |
| Raspberry Pi 4 | 8 GB | SD | 16+ GB |
| Raspberry Pi 5 | 16 GB | SD | 16+ GB |

### Connectivity & I/O

| Platform | WiFi | BLE | GPIO | UART | I2C | SPI | USB |
|:---------|:----:|:---:|-----:|-----:|----:|----:|:----|
| **COR24-TB** | - | - | 10 | 1 | BB | BB | Power |
| Arduino Uno R4 | - | - | 14 | 1 | 1 | 1 | CDC |
| ESP32-C3 | 4 | 5.0 | 19 | 2 | 1 | 3 | Yes |
| ESP32-C6 | 6 | 5.3 | 19 | 2 | 1 | 1 | Yes |
| ESP32-S3 | 4 | 5.0 | 45 | 3 | 2 | 4 | OTG |
| STM32F103 | - | - | 37 | 3 | 2 | 2 | Yes |
| Pi 5 | ac | 5.0 | 40 | 6 | 6 | 7 | 4x USB3 |

*BB = Bit-bang (software implementation)*

### Price & Power

| Platform | Voltage | Current | Sleep | Price |
|:---------|--------:|--------:|------:|------:|
| **COR24-TB** | 5V | 75 mA | N/A | $190 |
| Arduino Uno R4 | 5V | 100 mA | 100 uA | $27 |
| ESP32-C3 | 3.3V | 80 mA | 5 uA | $4 |
| ESP32-S3 | 3.3V | 100 mA | 7 uA | $7 |
| STM32F103 | 3.3V | 25 mA | 2 uA | $4 |
| 8051 | 5V | 20 mA | 1 uA | $2 |
| Pi Zero 2 | 5V | 300 mA | N/A | $15 |
| Pi 5 | 5V | 1500 mA | N/A | $80 |

---

## COR24 Unique Value Proposition

```mermaid
flowchart LR
    subgraph "What Makes COR24-TB Special"
        direction TB
        A[Full RTL Source Code] --> B[Modify CPU Design]
        B --> C[Add Custom Instructions]
        C --> D[Learn Architecture]
        D --> E[Build Emulators]
    end

    subgraph "Other Platforms"
        direction TB
        X[Binary Blob CPU] --> Y[Fixed Instruction Set]
        Y --> Z[No Modification Possible]
    end

    style A fill:#90EE90
    style B fill:#90EE90
    style C fill:#90EE90
    style D fill:#90EE90
    style E fill:#90EE90
```

---

## Historical Context

```mermaid
timeline
    title Computer Architecture Evolution
    section 8-bit Era
        1971 : Intel 4004 (4-bit)
        1974 : Intel 8080 (8-bit)
        1976 : MOS 6502
        1980 : Intel 8051
    section 16-bit Era
        1978 : Intel 8086
        1982 : Motorola 68000
    section 32-bit RISC
        1985 : ARM1
        1986 : MIPS R2000
        1992 : ARM7TDMI
    section Modern
        2005 : ARM Cortex-M
        2016 : ESP32
        2020 : RISC-V mainstream
        2024 : COR24 (24-bit open RISC)
```

---

## Summary Radar Chart

```mermaid
%%{init: {"themeVariables": {"pie1": "#90EE90", "pie2": "#87CEEB", "pie3": "#FFB6C1"}}}%%
pie showData
    title "COR24-TB Strengths (Score out of 10)"
    "Openness: 10" : 10
    "RAM Size: 8" : 8
    "Hackability: 9" : 9
    "Educational: 10" : 10
    "Performance: 4" : 4
    "Connectivity: 2" : 2
    "Ecosystem: 3" : 3
    "Value: 3" : 3
```

---

## Price Positioning

At **$190**, the COR24-TB is positioned as a premium educational/development tool:

| Platform | Price | Price Ratio vs COR24-TB |
|:---------|------:|:-----------------------:|
| ESP32-C3 | $4 | 47x cheaper |
| STM32 Blue Pill | $4 | 47x cheaper |
| Seeed XIAO | $7 | 27x cheaper |
| Arduino Uno R4 | $27 | 7x cheaper |
| Raspberry Pi 5 (8GB) | $80 | 2.4x cheaper |
| **COR24-TB** | **$190** | **Baseline** |

**Why the premium?** Low-volume production, hand assembly, niche market, and the unique value of a fully open-source CPU design.

---

## Files Generated

| File | Format | Purpose |
|:-----|:-------|:--------|
| `cor24-comparison.md` | Markdown | Original detailed analysis |
| `cor24-comparison-visual.md` | Markdown + Mermaid | This file with diagrams |
| `cor24-comparison.csv` | CSV/Spreadsheet | Raw data for Excel/Sheets |

---

## Sources

- [MakerLisp COR24-TB](https://www.makerlisp.com/cor24-test-board)
- [Arduino Documentation](https://docs.arduino.cc/)
- [Raspberry Pi Specifications](https://www.raspberrypi.com/products/)
- [ESP32 Comparison Guide](https://www.espboards.dev/blog/esp32-soc-options/)
- [STM32 Blue Pill](https://stm32-base.org/boards/STM32F103C8T6-Blue-Pill.html)
- [PIC32 Architecture](https://www.microchip.com/en-us/products/microcontrollers/32-bit-mcus/pic32m)
