import { invoke } from "@tauri-apps/api/core";

import type {
  DatabaseInfo,
  DayDetail,
  TripDetail,
  TripSummary,
} from "./types";

export async function selectDatabase(path: string): Promise<DatabaseInfo> {
  return invoke<DatabaseInfo>("select_database", { path });
}

export async function listTripSummaries(): Promise<TripSummary[]> {
  return invoke<TripSummary[]>("list_trip_summaries");
}

export async function getTripDetail(tripId: number): Promise<TripDetail> {
  return invoke<TripDetail>("get_trip_detail", { tripId });
}

export async function getDayTimeline(
  tripId: number,
  dayNumber: number,
): Promise<DayDetail> {
  return invoke<DayDetail>("get_day_timeline", { tripId, dayNumber });
}
