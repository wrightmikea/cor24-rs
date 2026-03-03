# COR24-TB Competitive Ranking Analysis

**Platforms Compared (19 total):**
COR24-TB, Arduino Uno R3, Arduino Uno R4, Arduino Nano ESP32, ESP32-C3, ESP32-C6, ESP32-H2, ESP32-S3, Seeed XIAO ESP32C3, Seeed XIAO ESP32S3, STM32F103 (Blue Pill), STM32F411 (Black Pill), PIC32MX, PIC32MZ, 8051 Classic, 8051 Modern, Raspberry Pi Zero 2 W, Raspberry Pi 4, Raspberry Pi 5

---

## Table 1: Ranked Characteristics (Higher is Better)

| Characteristic | COR24-TB Value | Rank | Distribution | Notes |
|:---------------|:---------------|:----:|:-------------|:------|
| **Clock Speed** | 101.6 MHz | 11 of 19 | 3 have 1000+ MHz, 1 has 200 MHz, 6 have 160-240 MHz, **1 has 101.6 MHz**, 2 have 96-100 MHz, 3 have 48-80 MHz, 3 have 12-16 MHz | Mid-tier; faster than classic MCUs, slower than modern ESP32/Pi |
| **RAM Size** | 1,028 KB | 7 of 19 | 3 have 512MB+, 3 have 8+ MB, **1 has ~1 MB**, 4 have 320-512 KB, 4 have 20-128 KB, 4 have <32 KB | Strong for MCU class; 1MB SRAM is generous |
| **Address Space** | 24-bit (16 MB) | 16 of 19 | 3 have 64-bit, 12 have 32-bit, **1 has 24-bit**, 3 have 16-bit | Unusual; larger than 8/16-bit, smaller than modern 32-bit |
| **Word Size** | 24-bit | 13 of 19 | 3 have 64-bit, 12 have 32-bit, **1 has 24-bit**, 3 have 8-bit | Unique; only 24-bit platform in comparison |
| **CPU Cores** | 1 | 13 of 19 | 3 have 4 cores, 3 have 2 cores, **13 have 1 core** | Tied with most MCUs |
| **GPIO Count** | 10 | 17 of 19 | 1 has 50, 1 has 45, 3 have 40, 1 has 37, 1 has 32, 3 have 19, 3 have 14, 2 have 11, **1 has 10**, 2 vary | Near bottom; minimal I/O expansion |
| **UART Count** | 1 | 13 of 19 | 3 have 6, 1 has 3, 6 have 2, **9 have 1** | Tied with basic MCUs |
| **I2C Interfaces** | 0 (bit-bang) | 19 of 19 | 3 have 6, 2 have 3, 5 have 2, 8 have 1, **1 has 0** | Lowest; requires software implementation |
| **SPI Interfaces** | 0 (bit-bang) | 19 of 19 | 1 has 7, 1 has 5, 2 have 4, 2 have 3, 6 have 2, 6 have 1, **1 has 0** | Lowest; requires software implementation |
| **Flash/ROM** | External | 18 of 19 | 1 has 2MB, 4 have 512KB-16MB, 8 have 64-256KB, 3 have 4-32KB, **1 external**, 2 use SD | No onboard flash; relies on external load |
| **USB Capability** | Power Only | 18 of 19 | 4 have USB 3.0, 6 have USB OTG/2.0, 5 have USB CDC, 2 have Micro USB, **1 power only**, 1 none | Cannot communicate over USB |
| **Instruction Count** | 32 | — | Varies widely by architecture | Simple RISC; easy to understand/emulate |
| **Register Count** | 8 | — | Ranges 1-16+ across architectures | Moderate; comparable to classic RISC |

---

## Table 2: Ranked Characteristics (Lower is Better)

| Characteristic | COR24-TB Value | Rank | Distribution | Notes |
|:---------------|:---------------|:----:|:-------------|:------|
| **Price (USD)** | $190 | 19 of 19 | 5 cost $2-7, 5 cost $10-20, 4 cost $27-35, 2 cost $55-80, 2 cost $80-100, **1 costs $190** | Most expensive; 47x more than ESP32-C3 |
| **Typical Current (mA)** | 75 | 12 of 19 | 3 draw 600-2000 mA, 1 draws 300 mA, 6 draw 80-100 mA, **1 draws 75 mA**, 4 draw 25-50 mA, 4 draw 20-35 mA | Mid-tier; FPGA overhead vs pure MCU |
| **Deep Sleep (µA)** | N/A | 19 of 19 | 3 have no sleep, 1 has 100 µA, 4 have 5-14 µA, 6 have 0.5-2 µA, **1 has no deep sleep**, 4 N/A | No low-power mode; FPGA always active |
| **Development Complexity** | High | 17 of 19 | 3 need full Linux setup, **1 needs FPGA toolchain**, 15 have simple Arduino/vendor IDE | Requires Lattice Diamond for HDL changes |
| **Ecosystem Size** | Very Small | 19 of 19 | 3 have massive (Linux), 6 have large (Arduino/ESP), 5 have medium (vendor), 4 have small, **1 very small** | New product; limited community/libraries |

---

## Table 3: Binary Features (Has / Doesn't Have)

| Feature | COR24-TB | Others with Feature | Others without Feature | Notes |
|:--------|:--------:|:--------------------|:-----------------------|:------|
| **WiFi** | ✗ | 12 of 18 have it | 6 of 18 lack it | No wireless; ESP32, Pi have WiFi |
| **Bluetooth/BLE** | ✗ | 11 of 18 have it | 7 of 18 lack it | No wireless; most modern boards have BLE |
| **Hardware I2C** | ✗ | 17 of 18 have it | 1 of 18 lacks it | Only 8051 Classic also lacks hardware I2C |
| **Hardware SPI** | ✗ | 17 of 18 have it | 1 of 18 lacks it | Only 8051 Classic also lacks hardware SPI |
| **USB Data** | ✗ | 17 of 18 have it | 1 of 18 lacks it | Only 8051 Classic also lacks USB |
| **Linux Support** | ✗ | 3 of 18 have it | 15 of 18 lack it | Only Pi boards run Linux |
| **RTOS Support** | ✗ | 14 of 18 have it | 4 of 18 lack it | FreeRTOS common on ESP32/STM32/PIC |
| **ADC** | ✗ | 17 of 18 have it | 1 of 18 lacks it | Most MCUs have analog input |
| **DAC** | ✗ | 8 of 18 have it | 10 of 18 lack it | Less common; Arduino R4, ESP32, Pi have it |
| **PWM** | ✗ | 18 of 18 have it | 0 of 18 lack it | COR24-TB is only platform without hardware PWM |
| **CAN Bus** | ✗ | 5 of 18 have it | 13 of 18 lack it | Automotive protocol; STM32, PIC, some Arduino |
| **Camera Interface** | ✗ | 5 of 18 have it | 13 of 18 lack it | ESP32-S3, Pi boards have CSI |
| **Thread/Zigbee** | ✗ | 2 of 18 have it | 16 of 18 lack it | Only ESP32-C6, H2 have 802.15.4 |
| **JTAG Debug** | ✓ | 10 of 18 have it | 8 of 18 lack it | Standard for FPGA; shared with STM32, PIC |
| **Open Source CPU RTL** | ✓ | 0 of 18 have it | 18 of 18 lack it | **Unique to COR24-TB** |
| **Modifiable ISA** | ✓ | 0 of 18 have it | 18 of 18 lack it | **Unique to COR24-TB** |
| **FPGA Reprogrammable** | ✓ | 0 of 18 have it | 18 of 18 lack it | **Unique to COR24-TB** |

---

## Table 4: Architecture Characteristics

| Characteristic | COR24-TB | Distribution Across All 19 Platforms | Notes |
|:---------------|:---------|:-------------------------------------|:------|
| **Bits (Data Width)** | 24-bit ✓ | 3 have 64-bit, 12 have 32-bit, **1 has 24-bit**, 3 have 8-bit | Only 24-bit platform |
| **Architecture Type** | RISC ✓ | 6 are ARM, 5 are RISC-V, 3 are Xtensa, 2 are MIPS, 2 are CISC (8051), **1 is custom RISC** | Unique custom design |
| **Instruction Encoding** | Variable (1-4 byte) ✓ | Varies; ARM has fixed 32-bit, x86/8051 variable, RISC-V mixed | Compact encoding |
| **Endianness** | Little ✓ | Most are little-endian | Standard |
| **Hardware Multiply** | Yes (24 cycles) ✓ | 16 have single-cycle, **1 has multi-cycle**, 2 lack multiply | Slower than modern; functional |
| **Hardware Divide** | No ✗ | 10 have it, **9 lack it** | Common limitation for simple CPUs |
| **FPU** | No ✗ | 8 have it, **11 lack it** | No floating-point hardware |
| **DSP Extensions** | No ✗ | 5 have it, **14 lack it** | No signal processing acceleration |
| **Memory Protection** | No ✗ | 6 have it, **13 lack it** | Simple flat memory model |
| **Cache** | No ✗ | 3 have L1/L2/L3, **16 lack cache** | Direct memory access; no caching |

---

## Summary: COR24-TB Ranking Overview

### Strengths (Top 33% Rankings)

| Metric | Rank | Comment |
|:-------|:----:|:--------|
| RAM Size | 7 of 19 | 1MB SRAM beats most MCUs |
| Open Source RTL | 1 of 19 | **Only platform with this** |
| Modifiable ISA | 1 of 19 | **Only platform with this** |
| FPGA Flexibility | 1 of 19 | **Only platform with this** |
| JTAG Debug | Top 50% | Standard debugging interface |

### Middle Rankings (33-66%)

| Metric | Rank | Comment |
|:-------|:----:|:--------|
| Clock Speed | 11 of 19 | Respectable 101.6 MHz |
| Power Consumption | 12 of 19 | Moderate ~75mA |
| UART | 13 of 19 | Basic serial I/O |
| Cores | 13 of 19 | Single core (common) |

### Weaknesses (Bottom 33% Rankings)

| Metric | Rank | Comment |
|:-------|:----:|:--------|
| Price | 19 of 19 | Most expensive at $190 |
| I2C Hardware | 19 of 19 | None (bit-bang only) |
| SPI Hardware | 19 of 19 | None (bit-bang only) |
| Deep Sleep | 19 of 19 | No low-power mode |
| Ecosystem | 19 of 19 | Smallest community |
| USB Capability | 18 of 19 | Power only |
| Flash Storage | 18 of 19 | External load required |
| GPIO Count | 17 of 19 | Only 10 pins |
| Address Space | 16 of 19 | 24-bit vs 32/64-bit standard |
| WiFi/BLE | N/A | Not present |

---

## Key Insight

The COR24-TB ranks **first in exactly three characteristics** that no other platform offers:
1. Open Source CPU RTL
2. Modifiable Instruction Set Architecture
3. FPGA-based reprogrammable hardware

These unique features justify its existence despite ranking **last or near-last** in:
- Price (most expensive)
- Peripheral hardware (no I2C/SPI/ADC/PWM)
- Connectivity (no WiFi/BLE/USB data)
- Ecosystem support (smallest community)

**Bottom Line:** COR24-TB is not a general-purpose dev board—it's a specialized educational and research platform for CPU architecture exploration, where its unique capabilities have no competition.
