import type { DesktopErrorPayload } from "../types";

interface ErrorBannerProps {
  error: DesktopErrorPayload;
}

export function ErrorBanner({ error }: ErrorBannerProps) {
  return (
    <div className="error-banner" role="alert">
      <strong>{error.code}</strong>
      <p>{error.message}</p>
    </div>
  );
}
