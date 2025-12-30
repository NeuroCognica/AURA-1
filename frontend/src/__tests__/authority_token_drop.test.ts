import { configureStore } from "@reduxjs/toolkit";
import authorityReducer, { applyCouncilEvent } from "../authority/authoritySlice";
import { authorityGate, AuthorityViolationError } from "../authority/authorityGate";

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

    function handleToken(token: string) {
      const state = store.getState().authority as any;

      try {
        authorityGate(state, { kind: "render_language" }, () => renderedTokens.push(token));
      } catch (err) {
        if (err instanceof AuthorityViolationError) {
          return;
        }
        throw err;
      }
    }

    handleToken("hello");
    expect(renderedTokens).toEqual(["hello"]);

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
      }) as any
    );

    handleToken("this");
    handleToken("should");
    handleToken("not");
    handleToken("render");

    expect(renderedTokens).toEqual(["hello"]);

    const state = store.getState().authority as any;
    expect(() => authorityGate(state, { kind: "send_prompt" }, () => {})).toThrow(AuthorityViolationError);

    expect(() => authorityGate(state, { kind: "submit_consent" }, () => {})).not.toThrow();
  });
});
