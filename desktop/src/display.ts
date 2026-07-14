/** Display helpers for read-only Desktop polish (v4.10.2). */

const WEEKDAYS = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"] as const;

/**
 * Format an ISO date (`YYYY-MM-DD`) as `Mon · Apr 26`.
 * Returns the original string if parsing fails.
 */
export function formatDayLabel(isoDate: string): string {
  const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(isoDate.trim());
  if (!match) {
    return isoDate;
  }
  const year = Number(match[1]);
  const month = Number(match[2]);
  const day = Number(match[3]);
  const date = new Date(Date.UTC(year, month - 1, day));
  if (
    date.getUTCFullYear() !== year ||
    date.getUTCMonth() !== month - 1 ||
    date.getUTCDate() !== day
  ) {
    return isoDate;
  }
  const weekday = WEEKDAYS[date.getUTCDay()];
  const monthName = date.toLocaleString("en-US", {
    month: "short",
    timeZone: "UTC",
  });
  return `${weekday} · ${monthName} ${day}`;
}

export function formatDateRange(
  start?: string | null,
  end?: string | null,
): string | null {
  if (start && end) {
    return `${start} — ${end}`;
  }
  if (start) {
    return start;
  }
  if (end) {
    return end;
  }
  return null;
}

export function formatMinutes(value?: number | null): string | null {
  if (value === null || value === undefined) {
    return null;
  }
  return `${value} min`;
}

export function nonEmpty(value?: string | null): string | null {
  if (value === null || value === undefined) {
    return null;
  }
  const trimmed = value.trim();
  return trimmed.length > 0 ? trimmed : null;
}
