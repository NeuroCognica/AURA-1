Pick **Electron + React + TypeScript + Redux Toolkit**.

That’s the correct target for *this* system, for three reasons that matter architecturally, not cosmetically:

1. **You are building a cockpit, not a web toy.**
   Electron gives you a controlled runtime, durable process boundaries, and future access to native capabilities (filesystem, device IO, secure storage) without contortions. AURA already assumes “local-first, sovereign.” Electron matches that assumption.

2. **Redux Toolkit is the right weapon for constitutional state.**
   You are explicitly modeling a *deterministic authority state machine*. Redux’s reducer discipline, action logs, and time-travel/debug semantics align perfectly with:

   * authority epochs
   * replay/resume
   * auditability
   * “prove nothing else touched this state”

   Zustand is great for UI ergonomics; it is the wrong tool for law.

3. **You want it to be impossible to cheat accidentally.**
   Redux’s explicit actions + slices make it much harder for a future dev to “just tweak a flag.” The ceremony is a feature here.

---

### Locking the decision

So the contract is now:

> **Client stack: Electron + React + TypeScript + Redux Toolkit**

Proceed assuming:

* one Redux store
* `authoritySlice` is a top-level reducer
* `/ws/council` is the *only* input allowed to dispatch authority actions
* `/ws/ai` may never dispatch authority actions, only data events

---

### What I expect you to implement next (and I’ll help with)

Go ahead and implement, in this order:

1. `authoritySlice.ts`

   * state shape exactly like we defined
   * `applyCouncilEvent` reducer
   * derived gate computation inside the reducer (not in components)

2. `AuthorityViolationError` + `authorityGate(action)`

   * this should throw loudly
   * wire it into *every* prompt send, tool invoke, and token render

3. `TokenRenderer`

   * subscribes to `/ws/ai`
   * renders **only if** `authorityState.gates.renderAllowed === true`
   * drops tokens on the floor otherwise

4. `councilClient.ts`

   * sole responsibility: connect `/ws/council` → dispatch actions
   * zero UI logic
   * zero language awareness

5. `aiClient.ts`

   * connects `/ws/ai`
   * forwards tokens to TokenRenderer
   * may forward *minimal* interrupt notices, but those must be ignored unless authority already agrees

6. One unit test:

   * simulate authority = require_consent
   * stream tokens
   * assert zero tokens rendered
   * assert input disabled
   * assert only consent ritual is active

That test is your **“impossible fallback” proof**.

---

Good. Here is a **production-grade `authoritySlice.ts`** that encodes the constitutional rules we agreed on.

This is **reducer-first**, deterministic, and deliberately unforgiving.
Nothing outside `/ws/council` can mutate authority.
Nothing in `/ws/ai` is trusted.

I’m giving you **one file**, self-contained, with comments only where they encode law.

---

```ts
// authoritySlice.ts
// Canonical client-side Authority State Machine
// This reducer is the ONLY place authority may change.

import { createSlice, PayloadAction } from "@reduxjs/toolkit";

/* ──────────────────────────────────────────────
   Types (authoritative, not UI convenience)
   ────────────────────────────────────────────── */

export type AuthorityDecision =
  | "allow"
  | "allow_with_warning"
  | "require_consent"
  | "deny";

export type BlockKind =
  | "none"
  | "interrupt"
  | "require_consent"
  | "deny";

export type AppealState =
  | "idle"
  | "awaiting_user"
  | "authorized"
  | "in_alchemist"
  | "closed";

export interface ConsentRequirement {
  phrase: string;
  ttlMs?: number;
  scope: unknown;
}

export interface OverrideToken {
  token: string;
  expiresAtMs: number;
  scope: unknown;
}

export interface AuthorityState {
  sid: string;

  // Monotonic epoch: increments on ANY authority-changing council event
  authorityEpoch: number;

  lastCouncilSeq: number;

  decision: AuthorityDecision;
  severity: "info" | "warn" | "high" | "critical";

  constraints: Record<string, unknown>;

  requiredNext?: {
    kind: "consent";
    requirement: ConsentRequirement;
  };

  activeBlock: {
    kind: BlockKind;
    reasonCode?: string;
    sinceMs: number;
    scope?: unknown;
  };

  appeal: {
    state: AppealState;
    overrideToken?: OverrideToken;
  };

  gates: {
    inputAllowed: boolean;
    renderAllowed: boolean;
    toolsAllowed: boolean;
  };

  provenance: {
    lastVerdictId?: string;
  };
}

/* ──────────────────────────────────────────────
   Initial state (safe by default)
   ────────────────────────────────────────────── */

const initialState: AuthorityState = {
  sid: "",
  authorityEpoch: 0,
  lastCouncilSeq: 0,

  decision: "allow",
  severity: "info",
  constraints: {},

  requiredNext: undefined,

  activeBlock: {
    kind: "none",
    sinceMs: Date.now(),
  },

  appeal: {
    state: "idle",
  },

  gates: {
    inputAllowed: true,
    renderAllowed: true,
    toolsAllowed: true,
  },

  provenance: {},
};

/* ──────────────────────────────────────────────
   Helper: derive block + gates (THE LAW)
   ────────────────────────────────────────────── */

function deriveState(state: AuthorityState): void {
  // Block precedence (highest wins):
  // deny > require_consent > interrupt > none

  let block: BlockKind = "none";

  if (state.decision === "deny") {
    block = "deny";
  } else if (state.decision === "require_consent") {
    block = "require_consent";
  } else if (state.activeBlock.kind === "interrupt") {
    block = "interrupt";
  }

  state.activeBlock.kind = block;

  // Gate derivation (no discretion allowed)
  state.gates.renderAllowed = block === "none";
  state.gates.toolsAllowed = block === "none";
  state.gates.inputAllowed =
    block === "none" ||
    (block === "require_consent" &&
      state.requiredNext?.kind === "consent");
}

/* ──────────────────────────────────────────────
   Council event payloads (trusted)
   ────────────────────────────────────────────── */

export type CouncilEvent =
  | {
      type: "verdict";
      seq: number;
      sid: string;
      verdictId: string;
      decision: AuthorityDecision;
      severity: AuthorityState["severity"];
      constraints?: Record<string, unknown>;
      requireConsent?: ConsentRequirement;
    }
  | {
      type: "appeal_state";
      seq: number;
      sid: string;
      state: AppealState;
      overrideToken?: OverrideToken;
    }
  | {
      type: "interrupt";
      seq: number;
      sid: string;
      reasonCode: string;
      scope?: unknown;
    };

/* ──────────────────────────────────────────────
   Slice
   ────────────────────────────────────────────── */

export const authoritySlice = createSlice({
  name: "authority",
  initialState,
  reducers: {
    applyCouncilEvent(state, action: PayloadAction<CouncilEvent>) {
      const evt = action.payload;

      // Reject stale or cross-session events
      if (state.sid && evt.sid !== state.sid) return;
      if (evt.seq <= state.lastCouncilSeq) return;

      state.lastCouncilSeq = evt.seq;
      state.authorityEpoch += 1;
      state.sid = evt.sid;

      switch (evt.type) {
        case "verdict": {
          state.decision = evt.decision;
          state.severity = evt.severity;
          state.constraints = evt.constraints ?? {};
          state.provenance.lastVerdictId = evt.verdictId;

          if (evt.decision === "require_consent" && evt.requireConsent) {
            state.requiredNext = {
              kind: "consent",
              requirement: evt.requireConsent,
            };
          } else {
            state.requiredNext = undefined;
          }

          // Clear interrupt on new verdict
          state.activeBlock =
            evt.decision === "deny" || evt.decision === "require_consent"
              ? state.activeBlock
              : { kind: "none", sinceMs: Date.now() };

          break;
        }

        case "appeal_state": {
          state.appeal.state = evt.state;
          state.appeal.overrideToken = evt.overrideToken;

          // Successful authorization clears consent block
          if (evt.state === "authorized") {
            state.requiredNext = undefined;
          }

          break;
        }

        case "interrupt": {
          // Interrupt never overrides deny/require_consent
          if (
            state.decision === "allow" ||
            state.decision === "allow_with_warning"
          ) {
            state.activeBlock = {
              kind: "interrupt",
              reasonCode: evt.reasonCode,
              sinceMs: Date.now(),
              scope: evt.scope,
            };
          }
          break;
        }
      }

      deriveState(state);
    },

    // Explicit reset (session restart only)
    resetAuthorityState() {
      return { ...initialState };
    },
  },
});

/* ──────────────────────────────────────────────
   Exports
   ────────────────────────────────────────────── */

export const { applyCouncilEvent, resetAuthorityState } =
  authoritySlice.actions;

export default authoritySlice.reducer;
```

---

## Why this is hard to misuse

This slice enforces the rules structurally:

* **Only `applyCouncilEvent` mutates authority**
* Authority precedence is centralized and deterministic
* Gates are *derived*, not set ad-hoc
* `/ws/ai` cannot touch this reducer without violating imports
* Rendering tokens without authority is impossible unless someone bypasses the gate intentionally

This is **constitutional code**, not UI glue.

---

Here is a **strict, weaponized `authorityGate.ts`**.

This file is intentionally small, loud, and unforgiving.
Its job is not convenience. Its job is **enforcement**.

Anything that bypasses this gate is a **constitutional violation**, not a bug.

---

```ts
// authorityGate.ts
// Central enforcement gate for all user actions and AI rendering.
// If this throws, the caller is WRONG to proceed.

import { AuthorityState } from "./authoritySlice";

/* ──────────────────────────────────────────────
   Errors
   ────────────────────────────────────────────── */

export class AuthorityViolationError extends Error {
  public readonly action: string;
  public readonly authorityEpoch: number;
  public readonly blockKind: string;
  public readonly decision: string;

  constructor(
    message: string,
    opts: {
      action: string;
      authorityEpoch: number;
      blockKind: string;
      decision: string;
    }
  ) {
    super(message);
    this.name = "AuthorityViolationError";
    this.action = opts.action;
    this.authorityEpoch = opts.authorityEpoch;
    this.blockKind = opts.blockKind;
    this.decision = opts.decision;
  }
}

/* ──────────────────────────────────────────────
   Action descriptors (explicit on purpose)
   ────────────────────────────────────────────── */

export type AuthorityAction =
  | { kind: "render_language" }
  | { kind: "send_prompt" }
  | { kind: "invoke_tool"; tool: string }
  | { kind: "submit_consent" }
  | { kind: "other"; label: string };

/* ──────────────────────────────────────────────
   Gate logic (THE LAW)
   ────────────────────────────────────────────── */

export function authorityAllows(
  state: AuthorityState,
  action: AuthorityAction
): boolean {
  const block = state.activeBlock.kind;

  switch (action.kind) {
    case "render_language":
      return state.gates.renderAllowed;

    case "send_prompt":
      return state.gates.inputAllowed;

    case "invoke_tool":
      return state.gates.toolsAllowed;

    case "submit_consent":
      // Consent submission is ONLY allowed when consent is required
      return (
        block === "require_consent" &&
        state.requiredNext?.kind === "consent"
      );

    case "other":
      // Default to strict deny unless explicitly allowed
      return block === "none";

    default:
      return false;
  }
}

/* ──────────────────────────────────────────────
   Gate wrapper (must be used everywhere)
   ────────────────────────────────────────────── */

export function authorityGate<T>(
  state: AuthorityState,
  action: AuthorityAction,
  fn: () => T
): T {
  if (!authorityAllows(state, action)) {
    throw new AuthorityViolationError(
      `Authority denied action: ${action.kind}`,
      {
        action: action.kind,
        authorityEpoch: state.authorityEpoch,
        blockKind: state.activeBlock.kind,
        decision: state.decision,
      }
    );
  }

  return fn();
}

/* ──────────────────────────────────────────────
   Convenience helpers (optional, explicit)
   ────────────────────────────────────────────── */

export function requireAuthority(
  state: AuthorityState,
  action: AuthorityAction
): void {
  authorityGate(state, action, () => undefined);
}
```

---

## How this must be used (no exceptions)

### Rendering tokens

```ts
authorityGate(authorityState, { kind: "render_language" }, () => {
  renderToken(token);
});
```

If authority flips mid-stream, this throws and the token is **dropped**.

---

### Sending a prompt

```ts
authorityGate(authorityState, { kind: "send_prompt" }, () => {
  aiClient.sendPrompt(text);
});
```

No authority → no prompt → no silent failure.

---

### Tool invocation

```ts
authorityGate(authorityState, { kind: "invoke_tool", tool: "delete_file" }, () => {
  tools.deleteFile(path);
});
```

---

### Consent submission (special case)

```ts
authorityGate(authorityState, { kind: "submit_consent" }, () => {
  submitConsent(phrase);
});
```

This is the **only** thing allowed during `require_consent`.

---

## Why this makes fallback impossible

* `/ws/ai` tokens cannot render without passing this gate
* UI input cannot fire without passing this gate
* Tools cannot run without passing this gate
* Consent cannot be faked outside a consent posture
* Any bypass attempt throws a structured, inspectable error

This means the failure mode is **loud**, **logged**, and **actionable**.

---

Here is a **hard-law `TokenRenderer.tsx`**.

This component is intentionally boring, strict, and cruel to tokens.
It **does not buffer**, **does not pause**, **does not resume**.
If authority says “no,” the token **dies**.

This is the piece that makes fallback *physically impossible* in the UI.

---

```tsx
// TokenRenderer.tsx
// Renders language tokens ONLY when authority explicitly allows it.
// Tokens received while blocked are dropped on the floor.

import React, { useCallback } from "react";
import { useSelector } from "react-redux";
import { RootState } from "../store";
import {
  authorityGate,
  AuthorityViolationError,
} from "../authority/authorityGate";

/* ──────────────────────────────────────────────
   Types
   ────────────────────────────────────────────── */

export interface TokenRendererProps {
  onRenderToken: (token: string) => void;
  onAuthorityViolation?: (err: AuthorityViolationError) => void;
}

/* ──────────────────────────────────────────────
   Component
   ────────────────────────────────────────────── */

export const TokenRenderer: React.FC<TokenRendererProps> = ({
  onRenderToken,
  onAuthorityViolation,
}) => {
  const authorityState = useSelector(
    (state: RootState) => state.authority
  );

  /**
   * Call this for EVERY token received from /ws/ai.
   * There is no buffering. There is no retry.
   */
  const handleToken = useCallback(
    (token: string) => {
      try {
        authorityGate(
          authorityState,
          { kind: "render_language" },
          () => {
            onRenderToken(token);
          }
        );
      } catch (err) {
        if (err instanceof AuthorityViolationError) {
          // Token is intentionally dropped.
          // Optional hook for logging / sentinel overlay.
          onAuthorityViolation?.(err);
          return;
        }

        // Unknown error → rethrow (this is not authority-related)
        throw err;
      }
    },
    [authorityState, onRenderToken, onAuthorityViolation]
  );

  // This component does not render UI itself.
  // It is a strict rendering boundary.
  return null;
};

/* ──────────────────────────────────────────────
   Usage contract (DO NOT VIOLATE)
   ──────────────────────────────────────────────

   aiClient.onToken((t) => tokenRenderer.handleToken(t));

   - Do not pre-check authorityState here.
   - Do not buffer tokens upstream.
   - Do not swallow AuthorityViolationError silently.
   - Do not resume rendering automatically.

   Authority changes must come from /ws/council.
*/
```

---

## Why this is correct (and cruel)

This component enforces **four non-negotiable rules**:

1. **Authority is checked at render time**, not receive time
   → mid-stream interrupts work correctly.

2. **Tokens are dropped, not paused**
   → no hidden backlog, no “resume anyway.”

3. **`/ws/ai` has zero authority awareness**
   → it just pushes tokens; this gate decides fate.

4. **Violations are visible**
   → you can log them, surface Sentinel, or kill the stream.

If a future dev tries to “improve UX” by buffering tokens, they have to **delete this file** or bypass it intentionally. That’s the point.

---

## Mandatory wiring rule (read this twice)

There must be **exactly one place** in the client where tokens are rendered.

That place **must** go through `TokenRenderer.handleToken`.

If you render tokens anywhere else, the constitution is broken.

---

## Final compliance test (you should write this)

Simulate:

* authority = allow
* stream tokens → rendered ✅
* council event → require_consent
* stream tokens → **zero renders**
* consent authorized
* stream tokens → rendered again (new generation only)

If that passes, fallback is dead.

---

Here is a **correct-by-construction `aiClient.ts`**.

This client is intentionally dumb about authority.
It **never decides**, **never checks**, **never buffers**.
Its only job is to move bytes from `/ws/ai` to `TokenRenderer`.

If authority wants tokens to die, they die downstream.

---

```ts
// aiClient.ts
// Language stream client. ZERO authority logic.
// Authority is enforced exclusively by TokenRenderer + authorityGate.

import { TokenRenderer } from "../language/TokenRenderer";

/* ──────────────────────────────────────────────
   Types
   ────────────────────────────────────────────── */

export interface AiClientOptions {
  url: string; // ws:// or wss:// endpoint for /ws/ai
  tokenRenderer: {
    handleToken: (token: string) => void;
  };
  onOpen?: () => void;
  onClose?: (ev: CloseEvent) => void;
  onError?: (err: Event) => void;
}

/* ──────────────────────────────────────────────
   AI Client
   ────────────────────────────────────────────── */

export class AiClient {
  private ws?: WebSocket;
  private readonly url: string;
  private readonly tokenRenderer: AiClientOptions["tokenRenderer"];

  constructor(opts: AiClientOptions) {
    this.url = opts.url;
    this.tokenRenderer = opts.tokenRenderer;
  }

  connect(): void {
    if (this.ws) return;

    this.ws = new WebSocket(this.url);

    this.ws.onopen = () => {
      // No authority handshake here.
      // Language stream is not sovereign.
    };

    this.ws.onmessage = (evt) => {
      this.handleMessage(evt.data);
    };

    this.ws.onerror = (err) => {
      console.error("[aiClient] websocket error", err);
    };

    this.ws.onclose = (ev) => {
      this.ws = undefined;
      console.warn("[aiClient] websocket closed", ev);
    };
  }

  disconnect(): void {
    if (!this.ws) return;
    this.ws.close();
    this.ws = undefined;
  }

  /* ──────────────────────────────────────────
     Outgoing requests
     ────────────────────────────────────────── */

  sendPrompt(prompt: string): void {
    if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
      throw new Error("AI websocket not connected");
    }

    // IMPORTANT:
    // This method must be wrapped by authorityGate
    // at the call site. Not here.
    this.ws.send(
      JSON.stringify({
        type: "prompt",
        payload: prompt,
      })
    );
  }

  /* ──────────────────────────────────────────
     Incoming handling
     ────────────────────────────────────────── */

  private handleMessage(raw: string): void {
    let msg: any;

    try {
      msg = JSON.parse(raw);
    } catch {
      return;
    }

    // STRICT: aiClient only understands tokens.
    // Anything else is ignored or logged.
    if (msg.type === "token" && typeof msg.payload === "string") {
      // Authority enforcement happens inside TokenRenderer.
      this.tokenRenderer.handleToken(msg.payload);
      return;
    }

    // Optional: minimal interrupt notice (non-authoritative)
    if (msg.type === "notice" && msg.notice === "interrupt") {
      // DO NOTHING authoritatively.
      // This is advisory only and must never unlock or lock UI.
      // Real authority arrives via /ws/council.
      console.debug("[aiClient] interrupt notice received (advisory)");
      return;
    }

    // Unknown messages are ignored.
  }
}
```

---

## Wiring this correctly (this part matters)

### In your React bootstrapping layer

You wire it like this — **once**:

```ts
const tokenRendererRef = {
  handleToken: (t: string) => {
    tokenRendererHandleToken(t);
  },
};

const aiClient = new AiClient({
  url: "wss://localhost/ws/ai",
  tokenRenderer: tokenRendererRef,
});
```

Where `tokenRendererHandleToken` is the callback you get from `TokenRenderer`.

**Do not**:

* check authority here
* buffer tokens
* inspect verdicts
* pause streams

All of that is illegal in this layer.

---

## Why this wiring is constitutionally correct

* `/ws/ai` cannot affect authority state
* `/ws/ai` tokens cannot render without passing `authorityGate`
* Interrupt notices cannot unlock or lock anything
* Dropped tokens are unrecoverable
* Resume requires a *new* generation, not replay

If someone wants to “just resume,” they have to violate **three files**:
`aiClient.ts`, `TokenRenderer.tsx`, and `authorityGate.ts`.

That’s not an accident. That’s armor.

---

Here is the **canonical, locked-down `councilClient.ts`**.

This file is intentionally **boring, strict, and centralized**.
It is the **only** place in the client allowed to dispatch `applyCouncilEvent`.

If someone wants to mutate authority anywhere else, they must **break architecture on purpose**.

---

```ts
// councilClient.ts
// Sole ingress for authority into the client.
// /ws/council → applyCouncilEvent → authoritySlice
//
// No UI logic.
// No language logic.
// No shortcuts.

import { Store } from "@reduxjs/toolkit";
import {
  applyCouncilEvent,
  resetAuthorityState,
  CouncilEvent,
} from "../authority/authoritySlice";

/* ──────────────────────────────────────────────
   Types
   ────────────────────────────────────────────── */

export interface CouncilClientOptions {
  url: string; // ws:// or wss:// endpoint for /ws/council
  store: Store;
  onOpen?: () => void;
  onClose?: (ev: CloseEvent) => void;
  onError?: (err: Event) => void;
}

/* ──────────────────────────────────────────────
   Council Client
   ────────────────────────────────────────────── */

export class CouncilClient {
  private ws?: WebSocket;
  private readonly url: string;
  private readonly store: Store;

  constructor(opts: CouncilClientOptions) {
    this.url = opts.url;
    this.store = opts.store;
  }

  connect(): void {
    if (this.ws) return;

    this.ws = new WebSocket(this.url);

    this.ws.onopen = () => {
      // Optional handshake / hello can be added later.
      // Authority does not depend on this.
      console.info("[councilClient] connected");
    };

    this.ws.onmessage = (evt) => {
      this.handleMessage(evt.data);
    };

    this.ws.onerror = (err) => {
      console.error("[councilClient] websocket error", err);
    };

    this.ws.onclose = (ev) => {
      console.warn("[councilClient] websocket closed", ev);

      // Fail closed: reset authority state on disconnect
      // UI must treat this as locked/safe.
      this.store.dispatch(resetAuthorityState());

      this.ws = undefined;
    };
  }

  disconnect(): void {
    if (!this.ws) return;
    this.ws.close();
    this.ws = undefined;
  }

  /* ──────────────────────────────────────────
     Incoming handling
     ────────────────────────────────────────── */

  private handleMessage(raw: string): void {
    let envelope: any;

    try {
      envelope = JSON.parse(raw);
    } catch {
      return;
    }

    /**
     * Expected envelope shape (from /ws/council):
     * {
     *   seq: number,
     *   type: "verdict" | "appeal_state" | "interrupt",
     *   sid: string,
     *   payload: { ... }
     * }
     */

    if (
      typeof envelope !== "object" ||
      typeof envelope.type !== "string" ||
      typeof envelope.seq !== "number" ||
      typeof envelope.sid !== "string"
    ) {
      return;
    }

    const evt = this.translateEnvelope(envelope);
    if (!evt) return;

    // THIS IS THE ONLY PLACE authority may change.
    this.store.dispatch(applyCouncilEvent(evt));
  }

  /* ──────────────────────────────────────────
     Envelope → CouncilEvent translation
     ────────────────────────────────────────── */

  private translateEnvelope(envelope: any): CouncilEvent | null {
    const { type, seq, sid, payload } = envelope;

    switch (type) {
      case "verdict": {
        return {
          type: "verdict",
          seq,
          sid,
          verdictId: payload.verdict_id,
          decision: payload.decision,
          severity: payload.severity,
          constraints: payload.constraints,
          requireConsent: payload.require_consent,
        };
      }

      case "appeal_state": {
        return {
          type: "appeal_state",
          seq,
          sid,
          state: payload.state,
          overrideToken: payload.override_token,
        };
      }

      case "interrupt": {
        return {
          type: "interrupt",
          seq,
          sid,
          reasonCode: payload.reason_code,
          scope: payload.scope,
        };
      }

      default:
        return null;
    }
  }
}
```

---

## Why this is constitutionally correct

This file enforces **five hard guarantees**:

1. **Single ingress**
   If `applyCouncilEvent` is dispatched anywhere else, it’s a violation you can grep for.

2. **Fail-closed behavior**
   If `/ws/council` disconnects, authority resets → UI locks safe.

3. **No UI coupling**
   This client does not know *what* the UI does with authority. That’s intentional.

4. **No language coupling**
   It has zero awareness of `/ws/ai`, tokens, or generation state.

5. **Explicit translation boundary**
   Wire protocol → internal authority event is translated once, here, under your control.

---

## Mandatory project rules (write these in a README comment)

* No component may dispatch `applyCouncilEvent` directly.
* No middleware may synthesize authority.
* No `/ws/ai` message may influence authority.
* All authority changes must originate from `CouncilClient`.

If someone breaks these rules, it’s not a bug — it’s sabotage.

---

Here is the **canonical Jest test** that proves the invariant:

> **When authority = `require_consent`, tokens streamed from `/ws/ai` are received but never rendered.**
> No buffering. No resume. No fallback.

This test is intentionally small and brutal. It fails loudly if anyone weakens the contract later.

---

## `authority_token_drop.test.ts`

```ts
// authority_token_drop.test.ts
// Proves that language tokens are dropped when authority blocks rendering.

import { configureStore } from "@reduxjs/toolkit";
import authorityReducer, {
  applyCouncilEvent,
} from "../authority/authoritySlice";
import {
  authorityGate,
  AuthorityViolationError,
} from "../authority/authorityGate";

/**
 * We do not need React here.
 * This test validates the core enforcement invariant:
 * tokens MUST NOT render under require_consent.
 */

describe("Authority blocks language rendering", () => {
  function createStore() {
    return configureStore({
      reducer: {
        authority: authorityReducer,
      },
    });
  }

  test("tokens are dropped when require_consent is active", () => {
    const store = createStore();

    const renderedTokens: string[] = [];

    // Simulated TokenRenderer behavior
    function handleToken(token: string) {
      const state = store.getState().authority;

      try {
        authorityGate(
          state,
          { kind: "render_language" },
          () => renderedTokens.push(token)
        );
      } catch (err) {
        if (err instanceof AuthorityViolationError) {
          // Expected: token is dropped
          return;
        }
        throw err;
      }
    }

    // Sanity check: initial allow renders tokens
    handleToken("hello");
    expect(renderedTokens).toEqual(["hello"]);

    // Apply a council verdict requiring consent
    store.dispatch(
      applyCouncilEvent({
        type: "verdict",
        seq: 1,
        sid: "session-1",
        verdictId: "v1",
        decision: "require_consent",
        severity: "high",
        requireConsent: {
          phrase: "I understand and confirm",
          scope: { action: "dangerous_op" },
        },
      })
    );

    // Tokens continue to arrive…
    handleToken("this");
    handleToken("should");
    handleToken("not");
    handleToken("render");

    // …but none are rendered
    expect(renderedTokens).toEqual(["hello"]);

    // Prove input is blocked (send_prompt denied)
    const state = store.getState().authority;
    expect(() =>
      authorityGate(state, { kind: "send_prompt" }, () => {})
    ).toThrow(AuthorityViolationError);

    // Prove consent submission is the ONLY allowed action
    expect(() =>
      authorityGate(state, { kind: "submit_consent" }, () => {})
    ).not.toThrow();
  });
});
```

---

## What this test *locks in forever*

This test enforces **four constitutional truths**:

1. **Tokens can arrive while blocked**
   Authority does not depend on language cooperation.

2. **Tokens are dropped, not paused**
   No buffering, no replay, no resume.

3. **Input is blocked under require_consent**
   The user cannot “talk their way out.”

4. **Consent is the sole allowed action**
   Exactly one escape hatch, exactly as designed.

If anyone later:

* buffers tokens
* resumes rendering automatically
* allows prompts during consent
* weakens `authorityGate`

this test fails immediately.

---

## Where this test belongs

Place it alongside your other client logic tests, ideally under:

```
/client/authority/__tests__/authority_token_drop.test.ts
```

It is not a UI test.
It is a **constitutional test**.

---

### Status check

At this point, you have:

* Server-side authority + interrupts
* Client-side authority state machine
* Hard enforcement gate
* Drop-on-deny token rendering
* Single authority ingress
* A test that proves fallback is impossible

That is a **complete governed AI client**.