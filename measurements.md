# Case Study Measurements

## Smart Home Distributed (average over 20 launches)

| Component      | Platform                | Setup (ns) | Setup Complete (ns) | Loop (tps) |
|----------------|-------------------------|------------|---------------------|------------|
| Door Control   | Rust                    | 1,151,000  | 24,997,555,000      | 13,051     |
| Door Control   | Arduino (without Ember) | -          | -                   | -          |
| Door Control   | Arduino (with Ember)    | -          | -                   | -          |
| Center Control | Rust                    | 3,325,000  | 25,633,137,000      | 1,860      |
| Center Control | Arduino (without Ember) | -          | -                   | -          |
| Center Control | Arduino (with Ember)    | -          | -                   | -          |
| Plant Control  | Rust                    | 3,091,000  | 24,226,499,000      | 2,224      |
| Plant Control  | Arduino (without Ember) | -          | -                   | -          |
| Plant Control  | Arduino (with Ember)    | -          | -                   | -          |

---

## Static vs Dynamic Dispatch

| Count Target | Platform                | Dispatch Type | Time (ns)       |
|--------------|-------------------------|---------------|-----------------|
| 6,000        | Rust                    | Static        | 106,724,000     |
| 6,000        | Rust                    | Dynamic       | 107,707,000     |
| 6,000        | Arduino (without Ember) | Static        | -               |
| 6,000        | Arduino (with Ember)    | Static        | -               |
| 600,000      | Rust                    | Static        | 10,633,518,000  |
| 600,000      | Rust                    | Dynamic       | 10,760,725,000  |
| 600,000      | Arduino (without Ember) | Static        | -               |
| 600,000      | Arduino (with Ember)    | Static        | -               |
| 6,000,000    | Rust                    | Static        | 106,331,643,000 |
| 6,000,000    | Rust                    | Dynamic       | 107,606,350,000 |
| 6,000,000    | Arduino (without Ember) | Static        | -               |
| 6,000,000    | Arduino (with Ember)    | Static        | -               |

---

## HTTP Server

| Platform                | Setup (ns)    | Setup Complete (ns) | Loop (tps) | Avg (µs) | 1% Low (µs) | 10% Low (µs) | 1% High (µs) | 10% High (µs) |
|-------------------------|---------------|---------------------|------------|----------|-------------|--------------|--------------|---------------|
| Rust (with Ember)       | 540,000       | 3,042,097,000       | 3,259      | 123,855  | 40,018      | 53,928       | 390,459      | 219,068       |
| Rust (without Ember)    | 2,227,104,000 | -                   | 37,340     | 117,028  | 50,403      | 74,684       | 310,216      | 173,505       |
| Arduino (without Ember) | -             | -                   | -          | -        | -           | -            | -            | -             |
| Arduino (with Ember)    | -             | -                   | -          | -        | -           | -            | -            | -             |

---

## Plant Monitoring System

| Platform                | Setup Peripherals (ns) | Setup Ember (ns) | Loop (tps) |
|-------------------------|------------------------|------------------|------------|
| Rust (with Ember)       | 646,000                | 2,723,000        | 1,761      |
| Arduino (without Ember) | -                      | -                | -          |
| Arduino (with Ember)    | -                      | -                | -          |

---

## Home Automation

| Platform                | Setup Peripherals (ns) | Setup Ember (ns) | Loop (tps) |
|-------------------------|------------------------|------------------|------------|
| Rust (with Ember)       | 549,000                | 1,169,000        | 4,989      |
| Arduino (without Ember) | -                      | -                | -          |
| Arduino (with Ember)    | -                      | -                | -          |

---

## Colour Combinator

| Platform                | Setup Peripherals (ns) | Setup Ember (ns) | Runtime (ns) |
|-------------------------|------------------------|------------------|--------------|
| Rust (with Ember)       | 92,000                 | 1,951,000        | 104,381,000  |
| Rust (without Ember)    | 72,000                 | -                | 167,000      |
| Arduino (without Ember) | -                      | -                | -            |
| Arduino (with Ember)    | -                      | -                | -            |

