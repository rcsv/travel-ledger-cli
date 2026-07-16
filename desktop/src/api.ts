import { invoke } from "@tauri-apps/api/core";

import type {
  CreateItineraryInput,
  CreateItineraryResult,
  CreateTripInput,
  CreateTripResult,
  DatabaseInfo,
  DayDetail,
  RestoreLastDatabaseResult,
  TripDetail,
  TripSummary,
} from "./types";

export async function createItinerary(
  input: CreateItineraryInput,
): Promise<CreateItineraryResult> {
  return invoke<CreateItineraryResult>("create_itinerary", { input });
}

export async function createTrip(
  input: CreateTripInput,
): Promise<CreateTripResult> {
  return invoke<CreateTripResult>("create_trip", { input });
}

export async function selectDatabase(path: string): Promise<DatabaseInfo> {
  return invoke<DatabaseInfo>("select_database", { path });
}

export async function restoreLastDatabase(): Promise<RestoreLastDatabaseResult> {
  return invoke<RestoreLastDatabaseResult>("restore_last_database");
}

export async function forgetDatabase(): Promise<void> {
  return invoke("forget_database");
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
