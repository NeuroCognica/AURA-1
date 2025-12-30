import React, { useCallback } from "react";
import { useSelector } from "react-redux";
import { AuthorityState } from "../authority/authoritySlice";
import { RootState } from "../store";
import {
  authorityGate,
  AuthorityViolationError,
} from "../authority/authorityGate";

export interface TokenRendererProps {
  onRenderToken: (token: string) => void;
  onAuthorityViolation?: (err: AuthorityViolationError) => void;
}

export const TokenRenderer: React.FC<TokenRendererProps> = ({
  onRenderToken,
  onAuthorityViolation,
}) => {
  const authorityState = useSelector((state: RootState) => state.authority as AuthorityState);

  const handleToken = useCallback(
    (token: string) => {
      try {
        authorityGate(authorityState, { kind: "render_language" }, () => {
          onRenderToken(token);
        });
      } catch (err) {
        if (err instanceof AuthorityViolationError) {
          onAuthorityViolation?.(err);
          return;
        }
        throw err;
      }
    },
    [authorityState, onRenderToken, onAuthorityViolation]
  );

  return null;
};

export default TokenRenderer;
