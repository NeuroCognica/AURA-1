import { Store } from "@reduxjs/toolkit";
import {
  applyCouncilEvent,
  resetAuthorityState,
  CouncilEvent,
} from "../authority/authoritySlice";

export interface CouncilClientOptions {
  url: string;
  store: Store;
  onOpen?: () => void;
  onClose?: (ev: CloseEvent) => void;
  onError?: (err: Event) => void;
}

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
      console.info("[councilClient] connected");
    };

    this.ws.onmessage = (evt) => {
      this.handleMessage(evt.data as string);
    };

    this.ws.onerror = (err) => {
      console.error("[councilClient] websocket error", err);
    };

    this.ws.onclose = (ev) => {
      console.warn("[councilClient] websocket closed", ev);
      this.store.dispatch(resetAuthorityState());
      this.ws = undefined;
    };
  }

  disconnect(): void {
    if (!this.ws) return;
    this.ws.close();
    this.ws = undefined;
  }

  private handleMessage(raw: string): void {
    let envelope: any;

    try {
      envelope = JSON.parse(raw);
    } catch {
      return;
    }

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

    this.store.dispatch(applyCouncilEvent(evt));
  }

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
