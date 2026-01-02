import React, { useCallback, useEffect, useState } from "react";
import { Provider } from "react-redux";
import store from "./store";
import { TokenRenderer } from "./language/TokenRenderer";
import { CouncilClient } from "./clients/councilClient";
import { AiClient } from "./clients/aiClient";
import SentinelOverlay from "./components/SentinelOverlay";
import { useTheme } from "./themes/ThemeContext";
import ArchetypeSelector from "./components/ArchetypeSelector";

const tokenListStyle: React.CSSProperties = {
  padding: 12,
  margin: 12,
  border: "1px solid #ddd",
  minHeight: 80,
};

export const App: React.FC = () => {
  const [tokens, setTokens] = useState<string[]>([]);

  // create clients
  useEffect(() => {
    const council = new CouncilClient({ url: "ws://127.0.0.1:8080/ws/council", store });
    const ai = new AiClient({ url: "ws://127.0.0.1:8080/ws/ai", tokenRenderer: { handleToken: (t) => tokenHandler(t) } });

    council.connect();
    ai.connect();

    return () => {
      council.disconnect();
      ai.disconnect();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const tokenHandler = useCallback((t: string) => {
    setTokens((s) => [...s, t]);
  }, []);

  const onAuthorityViolation = useCallback((err: any) => {
    // optional: log or surface
    console.warn("authority violation", err);
  }, []);

  return (
    <Provider store={store}>
      <div>
        <h1 style={{ marginTop: 8 }}>AURA Frontend Shell (dev)</h1>
        <ArchetypeSelector />
        <div className={`aura-panel ${
          (useTheme().current.layoutHints?.emphasize === "left" && "emphasize-left") ||
          (useTheme().current.layoutHints?.emphasize === "center" && "emphasize-center") ||
          (useTheme().current.layoutHints?.emphasize === "right" && "emphasize-right") ||
          ""
        }`} style={tokenListStyle}>
          <strong>Rendered Tokens</strong>
          <div>
            {tokens.map((t, i) => (
              <span key={i}>{t}</span>
            ))}
          </div>
        </div>

        <TokenRenderer onRenderToken={(t) => tokenHandler(t)} onAuthorityViolation={onAuthorityViolation} />

        <SentinelOverlay />
      </div>
    </Provider>
  );
};

export default App;
