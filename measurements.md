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

Static: 218532000 ns
Dynamic: 221725000 ns

Count till 600 000.

Static: 21769595000 ns
Dynamic: 21899339000 ns

Count till 6 000 000.

Static: 217688345000 ns
Dynamic: 218965758000 ns
