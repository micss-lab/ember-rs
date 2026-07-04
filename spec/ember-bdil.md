# `ember-bdil` Content Language Specification

**Version:** 0.1.0  
**Language identifier (FIPA `:language`):** `ember-bdil`  
**Version location:** Encoded in the binary content frame ([§2.3](#23-frame-layout)), not in the language identifier string.

---

## Part 1 — Language

### 1.1 Purpose

`ember-bdil` is a FIPA ACL content language for exchanging BDI belief state between agents. A message carries one **content expression** — a single literal representing a belief. The FIPA performative encodes the intent (add or remove the belief); the content language encodes the belief itself.

### 1.2 Versioning

Language versions follow Semantic Versioning (MAJOR.MINOR.PATCH):

| Change kind | Version bump |
|---|---|
| Incompatible grammar change | MAJOR |
| New content expression type | MINOR |
| Clarification, new pre-coded functor | PATCH |

Receivers must reject messages whose MAJOR version differs from their own. A higher MINOR or PATCH in the received message is a forward-compatibility signal; receivers skip unknown content expression types (see [§2.3](#23-frame-layout)).

### 1.3 Grammar (v0.1.0)

```
content-expr  ::= literal

literal       ::= negation? functor arguments?
negation      ::= '~'
functor       ::= identifier
arguments     ::= '(' term-list ')'
term-list     ::= term (',' term)*

term          ::= integer
               |  float
               |  string
               |  literal
               |  variable

identifier    ::= [a-z_][a-zA-Z0-9_]*
variable      ::= [A-Z_][a-zA-Z0-9_]*
integer       ::= '-'? [0-9]+
float         ::= '-'? [0-9]* '.' [0-9]+ exponent?
              |   '-'? [0-9]+             '.' exponent?
exponent      ::= [eE] [+-]? [0-9]+
string        ::= '"' char* '"'
char          ::= <any byte except 0x00 and 0x22 (")> | escape
escape        ::= '\\' ['"\\nrt]
```

**Future extensions** add new `content-expr` alternatives (e.g. `plan`, `rule`) without modifying existing productions. The grammar is an open set of named alternatives keyed by a version-stable identifier.

### 1.4 Variables and Scope

Variables begin with an uppercase letter or `_`. A variable in a literal is a placeholder for an unknown ground term.

**Scope rule:** All variables in a single message are **message-local**. A variable name refers to the same value wherever it appears within the same message. Variables are fully independent of any variables in the receiving agent's knowledge base — they never unify with knowledge-base variables unless the agent explicitly binds them after decoding.

**Negation:** `~` marks negation-as-failure (NAF), not classical negation. This matches AgentSpeak semantics.

**Ember implementation note:** The decoder assigns fresh `VariableId`s to incoming variables using a name→id map scoped to that decode call. Multiple occurrences of the same name receive the same `VariableId`. The map is discarded after decoding.

### 1.5 Performative Mapping (v0.1.0)

The BDI event generated on receipt is determined solely by the FIPA performative. The mapping is **normative and exhaustive** for v0.1.0. Every performative not listed is a **protocol error** when `:language` is `ember-bdil`; receivers must respond with `not-understood`.

| FIPA Performative | Trigger | Goal Kind | BDI Event |
|---|---|---|---|
| `inform` | Addition | — | `+belief(literal)` |
| `not-understood` | Deletion | — | `-belief(literal)` |

Future MINOR versions may add new valid performatives (e.g. for goal or query communication). Such additions do not affect the v0.1.0 mapping.

---

## Part 2 — Bit-Efficient Encoding

### 2.1 Design Principles

The encoding follows the FIPA bit-efficient representation philosophy:

1. Every known keyword or construct type is represented by a **single predefined byte from the code table** — no known symbol is ever transmitted as a raw string.
2. All structures are **self-delimiting**: null-terminated words, END-terminated argument lists, and fixed-size numeric fields. No length-of-payload headers.
3. All unused byte ranges are **reserved and documented** so future versions extend the code table without conflict.

### 2.2 Code Table

All bytes not listed below are reserved and must not appear in v0.1.0 frames.

#### Frame-level codes

| Code | Name | Payload | Meaning |
|---|---|---|---|
| `0x00` | `END` | — | Terminates the expression list |
| `0x40` | `VER_0_1_0` | — | Language version 0.1.0 (implied, no extra bytes) |
| `0x41` | `VER_EXPLICIT` | `[major: u8][minor: u8][patch: u8]` | Explicit version number |
| `0x42`–`0x4F` | *(reserved)* | — | Future well-known versions |

#### Content expression type codes

| Code | Name | Payload | Meaning |
|---|---|---|---|
| `0x10` | `EXPR_LIT+` | `[expr_body]` | Positive literal |
| `0x11` | `EXPR_LIT-` | `[expr_body]` | Negated literal |
| `0x12`–`0x1F` | *(reserved)* | — | Future expression types (require MINOR bump) |

Expression bodies are self-delimiting (see [§2.4](#24-expression-body-literal-0x10--0x11)). Receivers that encounter an unknown expression code must reject the frame.

#### Functor codes

| Code | Name | Payload | Meaning |
|---|---|---|---|
| `0x30` | `WORD` | `[bytes][0x00]` | Null-terminated identifier |
| `0x31`–`0x3F` | *(reserved)* | — | Future pre-coded well-known functors |

#### Term codes

| Code | Name | Payload | Meaning |
|---|---|---|---|
| `0x00` | `END` | — | End of argument list |
| `0x20` | `T_INT` | `[i32, 4 bytes, little-endian]` | 32-bit signed integer |
| `0x21` | `T_FLT` | `[f32, 4 bytes, little-endian, IEEE 754]` | 32-bit float |
| `0x22` | `T_STR` | `[u16 BE len][bytes]` | Arbitrary byte string |
| `0x23` | `T_LIT+` | `[nested_literal]` | Positive literal as term |
| `0x24` | `T_LIT-` | `[nested_literal]` | Negated literal as term |
| `0x25` | `T_VAR` | `[name-bytes][0x00]` | Variable by name (null-terminated) |
| `0x26`–`0x2F` | *(reserved)* | — | Future term types |

Term codes have no per-term length prefix. If a receiver encounters an unknown term code (0x26–0x2F), it must reject the frame.

### 2.3 Frame Layout

```
frame    = magic version expr* END

magic    = 0xCA 0xED
version  = VER_0_1_0
         | VER_EXPLICIT major minor patch
expr     = expr_code expr_body
END      = 0x00

major, minor, patch  u8 each
```

The magic bytes `0xCA 0xED` are a fixed two-byte prefix identifying an `ember-bdil` frame.

After the magic and version, the parser reads expressions until it encounters `0x00` (END). All expression bodies are self-delimiting; unknown expression codes are a decode error.

**v0.1.0 constraint:** A valid v0.1.0 frame contains exactly one expression (`EXPR_LIT+` or `EXPR_LIT-`).

### 2.4 Expression Body: Literal (0x10 / 0x11)

```
expr_body      = functor arg_list

functor        = WORD functor_bytes 0x00
functor_bytes  = <non-empty sequence of identifier bytes, no 0x00>

arg_list       = (term_code term_payload)* END

term_code      = T_INT | T_FLT | T_STR | T_LIT+ | T_LIT- | T_VAR
```

The expression code (0x10 / 0x11) encodes the negation of the top-level literal. There is no separate negation byte in the body.

### 2.5 Nested Literal Payload (T_LIT+ / T_LIT-)

A nested literal is encoded as `functor arg_list` — identical to `expr_body` but without an expression code. Negation is encoded in the term code (0x23 vs 0x24).

### 2.6 Variable Encoding (T_VAR)

Variable names are null-terminated and must be non-empty with no embedded 0x00 bytes. Multiple occurrences of the same name within one frame decode to the same variable.

### 2.7 Worked Examples

**`location(agent1, room3)`** — positive, two 0-arity literal args:

```
CA ED              magic
40                 VER_0_1_0
10                 EXPR_LIT+
  30  6C6F636174696F6E 00       WORD  "location\0"
  23                            T_LIT+
    30  61 67 65 6E 74 31 00    WORD  "agent1\0"
    00                          END
  23                            T_LIT+
    30  72 6F 6F 6D 33 00       WORD  "room3\0"
    00                          END
  00                            END
00                 END

Total: 33 bytes
```

**`at(robot, X)`** — positive, one literal arg + one variable:

```
CA ED  40
10
  30  61 74 00                     WORD  "at\0"
  23  30  72 6F 62 6F 74 00  00    T_LIT+ WORD "robot\0" END
  25  58 00                        T_VAR  "X\0"
  00
00

Total: 23 bytes
```

**`~faulty(sensor1)`** — negated, one literal arg:

```
CA ED  40
11
  30  66 61 75 6C 74 79 00               WORD  "faulty\0"
  23  30  73 65 6E 73 6F 72 31 00  00    T_LIT+ WORD "sensor1\0" END
  00
00
```

### 2.8 Extension Rules for Future Versions

**New expression type (MINOR bump):**
1. Assign a code from `0x12`–`0x1F`.
2. The body must be self-delimiting (all fields null-terminated, END-terminated, or fixed-size).
3. Document the `expr_body` format in the new version's spec section.

**New term type (MINOR bump):**
1. Assign a code from `0x26`–`0x2F`.
2. The payload must be self-delimiting (fixed size, or null-terminated).
3. Document the encoding in the new version's spec section.

**Pre-coded well-known functors (PATCH bump):**
1. Assign codes from `0x31`–`0x3F`.
2. No payload — the functor string is implied by the code.
3. Document the code→string mapping in the patch spec section.

### 2.9 Embedding in FIPA ACL

The `ember-bdil` binary frame is placed verbatim in the FIPA ACL `:content` field. The outer FIPA bit-efficient codec wraps it in `BIN_STR_16` or `BIN_STR_32`. The `:language` field is set to `ember-bdil`.

### 2.10 Rejection Rules

| Condition | Action |
|---|---|
| First two bytes ≠ `[0xCA, 0xED]` | Reject: not an `ember-bdil` frame |
| `VERSION_MAJOR` ≠ receiver's MAJOR | Reject: incompatible version |
| Functor is empty (WORD immediately followed by 0x00) | Reject: malformed functor |
| Variable name is empty or contains 0x00 | Reject: malformed variable |
| v0.1.0 frame contains ≠ 1 expression | Reject: malformed content |
| Unknown expression code (0x12–0x1F) | Reject: unrecognised expression |
| Unknown term code (0x26–0x2F) | Reject: unrecognised term |
| `:language ember-bdil` with performative other than `inform` or `disconfirm` | Respond `not-understood` |
