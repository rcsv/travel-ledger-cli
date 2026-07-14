export interface DesktopErrorPayload {
  code: string;
  message: string;
}

export interface DatabaseInfo {
  path: string;
  trip_count: number;
}

export type RestoreLastDatabaseResult =
  | { status: "restored"; database: DatabaseInfo }
  | { status: "not_found" }
  | { status: "invalid_cleared"; code: string; message: string };

export interface TripSummary {
  id: number;
  name: string;
  start_date?: string | null;
  end_date?: string | null;
  summary?: string | null;
  main_destination?: string | null;
  main_destination_country_code?: string | null;
  default_currency?: string | null;
  created_at: string;
  updated_at: string;
}

export interface DaySummary {
  id: number;
  trip_id: number;
  day_number: number;
  date: string;
  title: string;
  summary?: string | null;
}

export interface TripDetail {
  id: number;
  name: string;
  start_date?: string | null;
  end_date?: string | null;
  summary?: string | null;
  main_destination?: string | null;
  main_destination_country_code?: string | null;
  default_currency?: string | null;
  created_at: string;
  updated_at: string;
  days: DaySummary[];
}

export interface ItineraryDetail {
  id: number;
  trip_id: number;
  day_number: number;
  title: string;
  note?: string | null;
  start_time?: string | null;
  sort_order: number;
  duration_minutes?: number | null;
  travel_minutes?: number | null;
  location?: string | null;
  category?: string | null;
  created_at: string;
  updated_at: string;
}

export interface DayDetail {
  trip_id: number;
  trip_name: string;
  day_id: number;
  day_number: number;
  date: string;
  title: string;
  summary?: string | null;
  itineraries: ItineraryDetail[];
}

export function isDesktopError(value: unknown): value is DesktopErrorPayload {
  return (
    typeof value === "object" &&
    value !== null &&
    "code" in value &&
    "message" in value &&
    typeof (value as DesktopErrorPayload).code === "string" &&
    typeof (value as DesktopErrorPayload).message === "string"
  );
}

export function formatDateRange(
  start?: string | null,
  end?: string | null,
): string {
  if (start && end) {
    return `${start} — ${end}`;
  }
  if (start) {
    return start;
  }
  if (end) {
    return end;
  }
  return "Dates not set";
}

export function databaseFileName(path: string): string {
  const normalized = path.replace(/\\/g, "/");
  const parts = normalized.split("/");
  return parts[parts.length - 1] || path;
}
