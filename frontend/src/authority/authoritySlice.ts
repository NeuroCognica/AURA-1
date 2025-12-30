// Canonical client-side Authority State Machine (from code_blocks1.md)
import { createSlice, PayloadAction } from "@reduxjs/toolkit";

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

function deriveState(state: AuthorityState): void {
  let block: BlockKind = "none";

  if (state.decision === "deny") {
    block = "deny";
  } else if (state.decision === "require_consent") {
    block = "require_consent";
  } else if (state.activeBlock.kind === "interrupt") {
    block = "interrupt";
  }

  state.activeBlock.kind = block;

  state.gates.renderAllowed = block === "none";
  state.gates.toolsAllowed = block === "none";
  state.gates.inputAllowed =
    block === "none" ||
    (block === "require_consent" && state.requiredNext?.kind === "consent");
}

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

export const authoritySlice = createSlice({
  name: "authority",
  initialState,
  reducers: {
    applyCouncilEvent(state, action: PayloadAction<CouncilEvent>) {
      const evt = action.payload;

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

          state.activeBlock =
            evt.decision === "deny" || evt.decision === "require_consent"
              ? state.activeBlock
              : { kind: "none", sinceMs: Date.now() };

          break;
        }

        case "appeal_state": {
          state.appeal.state = evt.state;
          state.appeal.overrideToken = evt.overrideToken;

          if (evt.state === "authorized") {
            state.requiredNext = undefined;
          }

          break;
        }

        case "interrupt": {
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

    resetAuthorityState() {
      return { ...initialState };
    },
  },
});

export const { applyCouncilEvent, resetAuthorityState } =
  authoritySlice.actions;

export default authoritySlice.reducer;
