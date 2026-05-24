# Phase 38: hp41-core — XROM Framework + Clock/Date/Stopwatch/Alarm Core - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-24
**Phase:** 38-hp41-core-xrom-framework-clock-date-stopwatch-alarm-core
**Areas discussed:** Clock access in core, Control alarm scope, Stopwatch save/load, Alarm catalog shape

---

## Clock Access in Core

### Q1: How should hp41-core access the system clock?

| Option | Description | Selected |
|--------|-------------|----------|
| Direct SystemTime | Call std::time::SystemTime::now() directly in core. Precedent: core already uses Arc<AtomicBool> and Instant. | ✓ |
| Frontend-injected clock | Frontend sets a transient CalcState field before each dispatch(). Core stays pure. | |
| You decide | Let Claude pick. | |

**User's choice:** Direct SystemTime (Recommended)
**Notes:** Aligns with existing precedent (Arc<AtomicBool>, Instant transitive dep).

### Q2: Time offset model?

| Option | Description | Selected |
|--------|-------------|----------|
| Single i64 offset | One persistent CalcState field: time_offset_secs: i64. Simpler, fewer fields. | ✓ |
| Separate time/date offsets | Two fields: time_offset_secs + date_offset_days. More explicit. | |

**User's choice:** Single i64 offset (Recommended)
**Notes:** None.

### Q3: Local time vs UTC?

| Option | Description | Selected |
|--------|-------------|----------|
| OS local time | Use libc::localtime_r / GetLocalTime. Matches real HP-41CX behavior. | ✓ |
| UTC internally | Store/compute in UTC, only format to local for display. | |

**User's choice:** OS local time (Recommended)
**Notes:** Users see the same time as their system clock.

---

## Control Alarm Scope

### Q1: Should interrupting control alarms be deferred?

| Option | Description | Selected |
|--------|-------------|----------|
| Defer + document | Implement message + non-interrupting control alarms. Document interrupting as divergence. | ✓ |
| Attempt full implementation | Try to implement with re-entrancy. High risk. | |
| Stub with error | Store but return DATA ERROR when control alarm fires. | |

**User's choice:** Defer + document (Recommended)
**Notes:** Matches research recommendation, avoids re-entrancy risk.

### Q2: Non-interrupting control alarm behavior?

| Option | Description | Selected |
|--------|-------------|----------|
| XEQ on acknowledgment | Execute XEQ <stored_label> when user acknowledges. Faithful emulation. | ✓ |
| Display label only | Show label but don't auto-execute. | |

**User's choice:** XEQ on acknowledgment (Recommended)
**Notes:** Uses existing XEQ dispatch, no re-entrancy hazard.

---

## Stopwatch Save/Load

### Q1: Save behavior with running stopwatch?

| Option | Description | Selected |
|--------|-------------|----------|
| Freeze elapsed on save | Compute elapsed, store as persistent f64. Loads as STOPPED. | ✓ |
| Reset on load | Fully transient. Loads as zero. | |
| Resume on load | Store wall-clock timestamp, compute gap. Fragile. | |

**User's choice:** Freeze elapsed on save (Recommended)
**Notes:** Instant can't serialize; pretending to track time across sessions is misleading.

### Q2: Stopwatch CalcState representation?

| Option | Description | Selected |
|--------|-------------|----------|
| Enum + f64 + transient Instant | Three separate fields with clear persistent/transient split. | ✓ |
| Single Option<StopwatchState> | Bundled struct, harder to split serde attributes. | |
| You decide | Let Claude pick. | |

**User's choice:** Enum + f64 + transient Instant (Recommended)
**Notes:** Clear separation of persistent vs transient fields.

---

## Alarm Catalog Shape

### Q1: Alarm type representation?

| Option | Description | Selected |
|--------|-------------|----------|
| Typed enum | AlarmType enum: Message(String) + Control { label, interrupting }. Forward-compatible. | ✓ |
| Flat struct with optional fields | All fields directly on AlarmEntry. Less type-safe. | |
| You decide | Let Claude pick. | |

**User's choice:** Typed enum (Recommended)
**Notes:** Forward-compatible for interrupting alarms when eventually implemented.

### Q2: Past-due alarm detection trigger?

| Option | Description | Selected |
|--------|-------------|----------|
| After every dispatch | check_alarms() via drain pattern, same as print_buffer/event_buffer. | ✓ |
| Only during poll/setInterval | Check only during live-display ticks. | |
| Both | Check after dispatch AND during poll. | |

**User's choice:** After every dispatch (Recommended)
**Notes:** Matches TIME-ALM-08 requirement. Alarm notifications via event_buffer.

### Q3: Alarm catalog CalcState field shape?

| Option | Description | Selected |
|--------|-------------|----------|
| Direct Vec<AlarmEntry> | alarms: Vec<AlarmEntry> with #[serde(default)]. Matches existing patterns. | ✓ |
| AlarmCatalog newtype | Wrapper struct. More encapsulated but new pattern. | |

**User's choice:** Direct Vec<AlarmEntry> (Recommended)
**Notes:** Consistent with regs: Vec<HpNum>, programs: Vec<Program>.

### Q4: Repeat interval storage format?

| Option | Description | Selected |
|--------|-------------|----------|
| Seconds i64 | Total seconds. Simple arithmetic for rescheduling. | ✓ |
| HpNum HH.MMSSss | Original format. Requires conversion each reschedule. | |

**User's choice:** Seconds i64 (Recommended)
**Notes:** Convert once at XYZALM entry time, no drift risk.

---

## Claude's Discretion

None — all decisions made explicitly by the user.

## Deferred Ideas

- Interrupting control alarm execution (re-entrancy risk — documented divergence)
- Cycle-accurate crystal oscillator simulation (no user value)
- HP-IL alarm wake-up (no OFF state in emulator)
- Accuracy factor correction loop (host OS clock is NTP-synchronized)
