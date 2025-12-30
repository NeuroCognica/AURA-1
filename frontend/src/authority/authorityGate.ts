import { AuthorityState } from "./authoritySlice";

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

export type AuthorityAction =
  | { kind: "render_language" }
  | { kind: "send_prompt" }
  | { kind: "invoke_tool"; tool: string }
  | { kind: "submit_consent" }
  | { kind: "other"; label: string };

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
      return (
        block === "require_consent" &&
        state.requiredNext?.kind === "consent"
      );

    case "other":
      return block === "none";

    default:
      return false;
  }
}

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

export function requireAuthority(
  state: AuthorityState,
  action: AuthorityAction
): void {
  authorityGate(state, action, () => undefined);
}
