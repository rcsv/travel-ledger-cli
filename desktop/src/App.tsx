import { useCallback, useEffect, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";

import * as api from "./api";
import { ErrorBanner } from "./components/ErrorBanner";
import { EmptyState } from "./components/EmptyState";
import { ItineraryTimeline } from "./components/ItineraryTimeline";
import { SettingsPanel } from "./components/SettingsPanel";
import { TripDetailPanel } from "./components/TripDetailPanel";
import { TripList } from "./components/TripList";
import type {
  DayDetail,
  DesktopErrorPayload,
  TripDetail,
  TripSummary,
} from "./types";
import { databaseFileName, isDesktopError } from "./types";
import "./App.css";

type MainView = "trips" | "settings";

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
  const [bootstrapping, setBootstrapping] = useState(true);
  const [mainView, setMainView] = useState<MainView>("trips");
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
  const [restoreWarning, setRestoreWarning] =
    useState<DesktopErrorPayload | null>(null);

  const clearTripSelection = useCallback(() => {
    setSelectedTripId(null);
    setTripDetail(null);
    setSelectedDayNumber(null);
    setDayTimeline(null);
  }, []);

  const clearAllData = useCallback(() => {
    setDatabasePath(null);
    setTrips([]);
    clearTripSelection();
    setMainView("trips");
  }, [clearTripSelection]);

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

  useEffect(() => {
    let cancelled = false;

    async function bootstrap() {
      setBootstrapping(true);
      setRestoreWarning(null);
      try {
        const result = await api.restoreLastDatabase();
        if (cancelled) {
          return;
        }
        if (result.status === "restored") {
          setDatabasePath(result.database.path);
          setLoadingTrips(true);
          try {
            const summaries = await api.listTripSummaries();
            if (cancelled) {
              return;
            }
            setTrips(summaries);
            if (summaries.length > 0) {
              const firstId = summaries[0].id;
              setSelectedTripId(firstId);
              setLoadingDetail(true);
              setTripDetail(null);
              setSelectedDayNumber(null);
              setDayTimeline(null);
              try {
                const detail = await api.getTripDetail(firstId);
                if (cancelled) {
                  return;
                }
                setTripDetail(detail);
                if (detail.days.length > 0) {
                  const firstDay = detail.days[0].day_number;
                  setSelectedDayNumber(firstDay);
                  setLoadingTimeline(true);
                  try {
                    const timeline = await api.getDayTimeline(firstId, firstDay);
                    if (!cancelled) {
                      setDayTimeline(timeline);
                    }
                  } catch (err) {
                    if (!cancelled) {
                      setError(toDesktopError(err));
                      setDayTimeline(null);
                    }
                  } finally {
                    if (!cancelled) {
                      setLoadingTimeline(false);
                    }
                  }
                }
              } catch (err) {
                if (!cancelled) {
                  setError(toDesktopError(err));
                }
              } finally {
                if (!cancelled) {
                  setLoadingDetail(false);
                }
              }
            } else {
              clearTripSelection();
            }
          } catch (err) {
            if (!cancelled) {
              setError(toDesktopError(err));
              clearTripSelection();
              setTrips([]);
            }
          } finally {
            if (!cancelled) {
              setLoadingTrips(false);
            }
          }
        } else if (result.status === "invalid_cleared") {
          setRestoreWarning({
            code: result.code,
            message: result.message,
          });
          clearAllData();
        } else {
          clearAllData();
        }
      } catch (err) {
        if (!cancelled) {
          setError(toDesktopError(err));
          clearAllData();
        }
      } finally {
        if (!cancelled) {
          setBootstrapping(false);
        }
      }
    }

    void bootstrap();
    return () => {
      cancelled = true;
    };
    // Bootstrap once on mount; later Change/Forget use explicit handlers.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const pickDatabasePath = useCallback(async (): Promise<string | null> => {
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
      return null;
    }
    return Array.isArray(selected) ? selected[0] : selected;
  }, []);

  const handleOpenOrChangeDatabase = useCallback(async () => {
    setError(null);
    setRestoreWarning(null);
    const hadDatabase = databasePath !== null;
    const path = await pickDatabasePath();
    if (path === null) {
      return;
    }

    setLoadingTrips(true);
    try {
      const info = await api.selectDatabase(path);
      clearTripSelection();
      setTrips([]);
      setDatabasePath(info.path);
      await loadTrips();
      if (!hadDatabase) {
        setMainView("trips");
      }
    } catch (err) {
      setError(toDesktopError(err));
      if (!hadDatabase) {
        clearAllData();
      }
    } finally {
      setLoadingTrips(false);
    }
  }, [
    clearAllData,
    clearTripSelection,
    databasePath,
    loadTrips,
    pickDatabasePath,
  ]);

  const handleForgetDatabase = useCallback(async () => {
    const confirmed = window.confirm(
      "Stop remembering this database?\n\nOnly the saved path is cleared. Your SQLite database file stays on disk.",
    );
    if (!confirmed) {
      return;
    }
    setError(null);
    setRestoreWarning(null);
    try {
      await api.forgetDatabase();
      clearAllData();
    } catch (err) {
      setError(toDesktopError(err));
    }
  }, [clearAllData]);

  const handleSelectTrip = useCallback(
    async (tripId: number) => {
      setMainView("trips");
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

  const tripCountLabel =
    trips.length === 1 ? "1 trip" : `${trips.length} trips`;
  const settingsOpen = mainView === "settings";

  if (bootstrapping) {
    return (
      <div className="app-shell">
        <header className="app-header">
          <div>
            <h1>Travel Ledger Desktop</h1>
            <p className="app-subtitle">Developer preview · read-only</p>
          </div>
        </header>
        <main className="standalone-view">
          <EmptyState
            title="Starting…"
            message="Looking for the database you opened last time."
          />
        </main>
      </div>
    );
  }

  return (
    <div className="app-shell">
      <header className="app-header">
        <div>
          <h1>Travel Ledger Desktop</h1>
          <p className="app-subtitle">Developer preview · read-only</p>
          {databasePath ? (
            <p className="selected-db" title={databasePath}>
              Database: <strong>{databaseFileName(databasePath)}</strong>
            </p>
          ) : null}
        </div>
        {!databasePath ? (
          <div className="header-actions">
            <button
              type="button"
              className="primary-button"
              onClick={handleOpenOrChangeDatabase}
            >
              Open Database
            </button>
          </div>
        ) : null}
      </header>

      {restoreWarning || error ? (
        <div className="notice-area">
          {restoreWarning ? (
            <ErrorBanner
              error={{
                code: restoreWarning.code,
                message: `${restoreWarning.message} Open a database to continue.`,
              }}
            />
          ) : null}
          {error ? <ErrorBanner error={error} /> : null}
        </div>
      ) : null}

      {!databasePath ? (
        <main className="standalone-view">
          <EmptyState
            title="Open a Travel Ledger database"
            message="Choose an existing SQLite file (.db, .sqlite, or .sqlite3). After a successful open, the path can be remembered for next time. Nothing is created or deleted here."
          />
        </main>
      ) : settingsOpen ? (
        <main className="settings-view">
          <SettingsPanel
            databasePath={databasePath}
            onChangeDatabase={handleOpenOrChangeDatabase}
            onForgetDatabase={handleForgetDatabase}
            onBackToTrips={() => setMainView("trips")}
          />
        </main>
      ) : (
        <div className="app-body">
          <aside className="sidebar" aria-label="Trip list sidebar">
            <div className="sidebar-scroll">
              <div className="sidebar-header">
                <h2>Trips</h2>
                {!loadingTrips ? (
                  <p className="trip-count" aria-live="polite">
                    {tripCountLabel}
                  </p>
                ) : null}
              </div>
              <TripList
                trips={trips}
                selectedTripId={selectedTripId}
                loading={loadingTrips}
                onSelect={handleSelectTrip}
              />
            </div>
            <div className="sidebar-footer">
              <button
                type="button"
                className="nav-settings"
                onClick={() => setMainView("settings")}
              >
                Settings
              </button>
            </div>
          </aside>

          <main className="detail-pane">
            <TripDetailPanel
              trip={tripDetail}
              selectedDayNumber={selectedDayNumber}
              loading={loadingDetail}
              onSelectDay={handleSelectDay}
            >
              <ItineraryTimeline
                items={dayTimeline?.itineraries ?? null}
                loading={loadingTimeline}
              />
            </TripDetailPanel>
          </main>
        </div>
      )}
    </div>
  );
}
