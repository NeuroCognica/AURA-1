import React from "react";
import { useSelector } from "react-redux";
import { RootState } from "../store";

export const SentinelOverlay: React.FC = () => {
  const auth = useSelector((s: RootState) => s.authority);

  const blocked = auth.activeBlock.kind !== "none";
  const reason = auth.activeBlock.reasonCode ?? auth.decision;

  if (!blocked) return null;

  return (
    <div style={{
      position: "fixed",
      top: 0,
      left: 0,
      right: 0,
      bottom: 0,
      background: "rgba(0,0,0,0.5)",
      color: "white",
      display: "flex",
      alignItems: "center",
      justifyContent: "center",
      zIndex: 9999,
    }}>
      <div style={{padding: 24, maxWidth: 640}}>
        <h2>Sentinel Notice</h2>
        <p>Authority blocked rendering: <strong>{reason}</strong></p>
        <p>Severity: {auth.severity}</p>
      </div>
    </div>
  );
};

export default SentinelOverlay;
