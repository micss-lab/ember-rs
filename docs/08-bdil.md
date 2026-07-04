# 8. The `ember-bdil` Content Language

BDI agents share belief state with one another using a dedicated FIPA ACL content language. In the
Ember source this language is identified by the FIPA `:language` string **`ember-bdil`** and carried
by the `Content::Bdil` variant; the full wire-format specification lives in
[`spec/ember-bdil.md`](../spec/ember-bdil.md). This page is a practical summary: the spec is
authoritative for anyone implementing an interoperable encoder/decoder.

## 8.1 Purpose

The language carries **one belief** per message: a single *content expression* that is a literal. The
FIPA **performative** encodes the intent (assert or retract the belief); the content encodes the
belief itself. This keeps the payload tiny: important on links like ESP-NOW.

| FIPA Performative | Effect on receiver          |
| ----------------- | --------------------------- |
| `inform`          | Add the belief (`+belief`)  |
| `disconfirm`      | Retract the belief (`-belief`) |

From agent code you never build these frames by hand: you use the `.send` built-in action described
in [BDI Agents §7.13](./07-bdi-agents.md#713-inter-agent-belief-sharing). This page is for
understanding what goes on the wire.

## 8.2 Grammar (v0.1.0)

A content expression is a single literal:

```
content-expr ::= literal
literal      ::= "~"? functor arguments?
arguments    ::= "(" term ("," term)* ")"
term         ::= integer | float | string | literal | variable
```

Identifiers (functors) start lowercase; variables start uppercase or `_`. `~` denotes
negation-as-failure, matching AgentSpeak semantics. All variables in a message are **message-local**:
a name refers to the same value throughout one message and never unifies with the receiver's own
variables unless the receiving agent explicitly binds it after decoding.

## 8.3 Bit-efficient encoding

The wire format follows the FIPA bit-efficient philosophy:

1. Every known keyword/construct is a **single predefined byte**: no known symbol is sent as a raw
   string.
2. All structures are **self-delimiting**: null-terminated words, `END`-terminated argument lists,
   fixed-size numbers. There are no length-of-payload headers.
3. Unused byte ranges are **reserved** so future versions extend the code table without conflict.

### Frame layout

```
frame   = magic version expr* END
magic   = 0xCA 0xED
version = VER_0_1_0 (0x40)  |  VER_EXPLICIT (0x41) major minor patch
END     = 0x00
```

A valid v0.1.0 frame contains **exactly one** expression.

### Code table (v0.1.0)

| Code        | Name           | Meaning                                        |
| ----------- | -------------- | ---------------------------------------------- |
| `0x00`      | `END`          | Terminates an expression list / argument list  |
| `0x40`      | `VER_0_1_0`    | Implicit version 0.1.0                         |
| `0x41`      | `VER_EXPLICIT` | Explicit `major.minor.patch` (3 bytes)         |
| `0x10`      | `EXPR_LIT+`    | Positive top-level literal                     |
| `0x11`      | `EXPR_LIT-`    | Negated top-level literal                      |
| `0x30`      | `WORD`         | Null-terminated functor identifier             |
| `0x20`      | `T_INT`        | 32-bit signed integer, little-endian           |
| `0x21`      | `T_FLT`        | 32-bit IEEE-754 float, little-endian           |
| `0x22`      | `T_STR`        | `u16` big-endian length + bytes                |
| `0x23`      | `T_LIT+`       | Positive literal as an argument                |
| `0x24`      | `T_LIT-`       | Negated literal as an argument                 |
| `0x25`      | `T_VAR`        | Variable by name, null-terminated              |

All other bytes in these ranges are reserved. Encountering an unknown expression or term code is a
decode error.

### Worked example

`location(agent1, room3)` encodes to 33 bytes:

```
CA ED              magic
40                 VER_0_1_0
10                 EXPR_LIT+
  30 6C6F636174696F6E 00    WORD "location\0"
  23 30 616765...31 00 00   T_LIT+ WORD "agent1\0" END
  23 30 726F6F6D33 00 00    T_LIT+ WORD "room3\0" END
  00                        END (arguments)
00                 END (frame)
```

See [`spec/ember-bdil.md` §2.7](../spec/ember-bdil.md#27-worked-examples) for more worked examples,
including variables and negation.

## 8.4 Embedding in FIPA ACL

The binary frame is placed verbatim in the FIPA ACL `:content` field; the outer FIPA bit-efficient
codec wraps it as a binary string. The `:language` field identifies the language (`ember-bdil`).
Ember represents a decoded frame as `Content::Bdil(BdilContent::Literal(_))`.

## 8.5 Versioning and rejection

The language uses semantic versioning; receivers reject a frame whose **major** version differs from
their own but tolerate higher minor/patch (forward-compatibility). Frames are also rejected for bad
magic bytes, empty functors, malformed variables, a wrong expression count, or unknown codes. A
`:language ember-bdil` message with a performative other than the supported ones is answered with
`not-understood`. The full rejection table is in [`spec/ember-bdil.md` §2.10](../spec/ember-bdil.md#210-rejection-rules).

## 8.6 Extending the language

Future versions add new expression or term types by claiming reserved codes, always keeping bodies
self-delimiting, and documenting the addition in a new spec section. New expression/term types bump
the **minor** version; new pre-coded well-known functors bump the **patch** version. This lets the
belief-sharing vocabulary grow (e.g. to plans or rules) without breaking existing agents.

## 8.7 Next

- [Communication Channels](./09-communication-channels.md): how these frames actually cross devices.
