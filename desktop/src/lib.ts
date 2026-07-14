import {
  getDayTimeline,
  getTripDetail,
  listTripSummaries,
  selectDatabase,
} from "./api";
import type {
  DatabaseInfo,
  DayDetail,
  TripDetail,
  TripSummary,
} from "./types";

export const desktopApi = {
  selectDatabase,
  listTripSummaries,
  getTripDetail,
  getDayTimeline,
};

export type {
  DatabaseInfo,
  DayDetail,
  TripDetail,
  TripSummary,
};

export { isDesktopError } from "./types";
export {
  formatDateRange,
  formatDayLabel,
  formatMinutes,
  nonEmpty,
} from "./display";
