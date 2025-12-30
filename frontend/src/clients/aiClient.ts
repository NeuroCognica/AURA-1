export interface AiClientOptions {
  url: string;
  tokenRenderer: {
    handleToken: (token: string) => void;
  };
  onOpen?: () => void;
  onClose?: (ev: CloseEvent) => void;
  onError?: (err: Event) => void;
}

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
    };

    this.ws.onmessage = (evt) => {
      this.handleMessage(evt.data as string);
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

  sendPrompt(prompt: string): void {
    if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
      throw new Error("AI websocket not connected");
    }

    this.ws.send(
      JSON.stringify({
        type: "prompt",
        payload: prompt,
      })
    );
  }

  private handleMessage(raw: string): void {
    let msg: any;

    try {
      msg = JSON.parse(raw);
    } catch {
      return;
    }

    if (msg.type === "token" && typeof msg.payload === "string") {
      this.tokenRenderer.handleToken(msg.payload);
      return;
    }

    if (msg.type === "notice" && msg.notice === "interrupt") {
      console.debug("[aiClient] interrupt notice received (advisory)");
      return;
    }
  }
}
