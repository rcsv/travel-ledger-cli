import { useCallback, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";

import * as api from "./api";
import { ErrorBanner } from "./components/ErrorBanner";
import { EmptyState } from "./components/EmptyState";
import { ItineraryTimeline } from "./components/ItineraryTimeline";
import { TripDetailPanel } from "./components/TripDetailPanel";
import { TripList } from "./components/TripList";
import type {
  DayDetail,
  DesktopErrorPayload,
  TripDetail,
  TripSummary,
} from "./types";
import { isDesktopError } from "./types";
import "./App.css";

function toDesktopError(error: unknown): DesktopErrorPayload {
  if (isDesktopError(error)) {
    return error;
  }
  return {
    code: "STORAGE_FAILURE",
    message: error instanceof Error ? error.message : String(error),
  };
}

export default function App() {
  const [databasePath, setDatabasePath] = useState<string | null>(null);
  const [trips, setTrips] = useState<TripSummary[]>([]);
  const [selectedTripId, setSelectedTripId] = useState<number | null>(null);
  const [tripDetail, setTripDetail] = useState<TripDetail | null>(null);
  const [selectedDayNumber, setSelectedDayNumber] = useState<number | null>(
    null,
  );
  const [dayTimeline, setDayTimeline] = useState<DayDetail | null>(null);
  const [loadingTrips, setLoadingTrips] = useState(false);
  const [loadingDetail, setLoadingDetail] = useState(false);
  const [loadingTimeline, setLoadingTimeline] = useState(false);
  const [error, setError] = useState<DesktopErrorPayload | null>(null);

  const clearTripSelection = useCallback(() => {
    setSelectedTripId(null);
    setTripDetail(null);
    setSelectedDayNumber(null);
    setDayTimeline(null);
  }, []);

  const loadTimeline = useCallback(async (tripId: number, dayNumber: number) => {
    setLoadingTimeline(true);
    try {
      const timeline = await api.getDayTimeline(tripId, dayNumber);
      setDayTimeline(timeline);
    } catch (err) {
      setError(toDesktopError(err));
      setDayTimeline(null);
    } finally {
      setLoadingTimeline(false);
    }
  }, []);

  const loadTripDetail = useCallback(
    async (tripId: number) => {
      setLoadingDetail(true);
      setTripDetail(null);
      setSelectedDayNumber(null);
      setDayTimeline(null);
      try {
        const detail = await api.getTripDetail(tripId);
        setTripDetail(detail);
        if (detail.days.length > 0) {
          const firstDay = detail.days[0].day_number;
          setSelectedDayNumber(firstDay);
          await loadTimeline(tripId, firstDay);
        }
      } catch (err) {
        setError(toDesktopError(err));
      } finally {
        setLoadingDetail(false);
      }
    },
    [loadTimeline],
  );

  const loadTrips = useCallback(async () => {
    setLoadingTrips(true);
    try {
      const summaries = await api.listTripSummaries();
      setTrips(summaries);
      if (summaries.length > 0) {
        const firstId = summaries[0].id;
        setSelectedTripId(firstId);
        await loadTripDetail(firstId);
      } else {
        clearTripSelection();
      }
    } catch (err) {
      setError(toDesktopError(err));
      clearTripSelection();
      setTrips([]);
    } finally {
      setLoadingTrips(false);
    }
  }, [clearTripSelection, loadTripDetail]);

  const handleOpenDatabase = useCallback(async () => {
    setError(null);
    const selected = await open({
      multiple: false,
      directory: false,
      filters: [
        {
          name: "SQLite Database",
          extensions: ["db", "sqlite", "sqlite3"],
        },
      ],
    });

    if (selected === null) {
      return;
    }

    const path = Array.isArray(selected) ? selected[0] : selected;
    setLoadingTrips(true);
    clearTripSelection();
    setTrips([]);

    try {
      const info = await api.selectDatabase(path);
      setDatabasePath(info.path);
      await loadTrips();
    } catch (err) {
      setError(toDesktopError(err));
      setDatabasePath(null);
      clearTripSelection();
      setTrips([]);
    } finally {
      setLoadingTrips(false);
    }
  }, [clearTripSelection, loadTrips]);

  const handleSelectTrip = useCallback(
    async (tripId: number) => {
      if (tripId === selectedTripId) {
        return;
      }
      setError(null);
      setSelectedTripId(tripId);
      await loadTripDetail(tripId);
    },
    [loadTripDetail, selectedTripId],
  );

  const handleSelectDay = useCallback(
    async (dayNumber: number) => {
      if (!selectedTripId || dayNumber === selectedDayNumber) {
        return;
      }
      setError(null);
      setSelectedDayNumber(dayNumber);
      setDayTimeline(null);
      await loadTimeline(selectedTripId, dayNumber);
    },
    [loadTimeline, selectedDayNumber, selectedTripId],
  );

  return (
    <div className="app-shell">
      <header className="app-header">
        <div>
          <h1>Travel Ledger Desktop</h1>
          <p className="app-subtitle">Developer preview · read-only</p>
        </div>
        <button type="button" className="primary-button" onClick={handleOpenDatabase}>
          Open Travel Ledger Database
        </button>
      </header>

      {error ? <ErrorBanner error={error} /> : null}

      {!databasePath ? (
        <EmptyState
          title="Open a Travel Ledger database"
          message="Use the button above to choose an existing SQLite database (.db, .sqlite, .sqlite3)."
        />
      ) : (
        <div className="app-body">
          <aside className="sidebar" aria-label="Trip list sidebar">
            <div className="sidebar-header">
              <h2>Trips</h2>
              <p className="db-path" title={databasePath}>
                {databasePath}
              </p>
            </div>
            <TripList
              trips={trips}
              selectedTripId={selectedTripId}
              loading={loadingTrips}
              onSelect={handleSelectTrip}
            />
          </aside>

          <main className="detail-pane">
            <TripDetailPanel
              trip={tripDetail}
              selectedDayNumber={selectedDayNumber}
              loading={loadingDetail}
              onSelectDay={handleSelectDay}
            />
            <ItineraryTimeline
              items={dayTimeline?.itineraries ?? []}
              loading={loadingTimeline}
              dayNumber={selectedDayNumber}
            />
          </main>
        </div>
      )}
    </div>
  );
}
