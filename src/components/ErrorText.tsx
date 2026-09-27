import type { ReactNode } from "react";

function ErrorText({ children }: { children: ReactNode }) {
  return <p className="error-text">{children}</p>;
}

export default ErrorText;
