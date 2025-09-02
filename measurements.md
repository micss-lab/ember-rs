# Case study measurements

## Smart home distributed (average over 20 launches)

Door Control:
    - Setup: 1151000 nanoseconds
    - Setup complete: 24997555000 nanoseconds
    - Loop: 13051 tps (average)

Center Control:
    - Setup: 3325000 nanoseconds
    - Setup complete: 25633137000 nanoseconds
    - Loop: 1860 tps (average)

Plant Control:
    - Setup: 3091000 nanoseconds
    - Setup complete: 24226499000 nanoseconds
    - Loop: 2224 tps (average)

## Static vs Dynamic dispatch

Count till 6 000.

Static: 106724000 ns
Dynamic: 107707000 ns

Count till 600 000.

Static: 10633518000 ns
Dynamic: 10760725000 ns

Count till 6 000 000.

Static: 106331643000 ns
Dynamic: 107606350000 ns

## HTTP Server

With ember:

Setup: 540000 ns
Setup Complete: 3042097000 ns
Loop: 3259 tps

average: 123855 µs
1% low: 40018 µs
10% low: 53928 µs
1% high: 390459 µs
10% high: 219068 µs

Without Ember:

Setup: 2227104000 ns
Loop: 37340 tps

average: 117028 µs
1% low: 50403 µs
10% low: 74684 µs
1% high: 310216 µs
10% high: 173505 µs

## Plant Monitoring System

Setup peripherals: 646000 ns
Setup ember: 2723000 ns
Loop: 1761 tps

## Home Automation

Setup peripherals: 549000 ns
Setup ember: 1169000 ns
Loop: 4989 tps
