import { useCallback, useEffect, useRef, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";

import * as api from "./api";
import { ErrorBanner } from "./components/ErrorBanner";
import { EmptyState } from "./components/EmptyState";
import { ItineraryQuickAddForm } from "./components/ItineraryQuickAddForm";
import { ItineraryTimeline } from "./components/ItineraryTimeline";
import { SettingsPanel } from "./components/SettingsPanel";
import { TripCreateForm } from "./components/TripCreateForm";
import { TripDetailPanel } from "./components/TripDetailPanel";
import { TripList } from "./components/TripList";
import type {
  CreateItineraryInput,
  CreateTripInput,
  DayDetail,
  DesktopErrorPayload,
  TripDetail,
  TripSummary,
} from "./types";
import { databaseFileName, isDesktopError } from "./types";
import "./App.css";

type MainView = "trips" | "settings";
type WorkspaceMode = "view" | "create";
type ItineraryComposerTarget = { tripId: number; dayNumber: number };

function sameComposerTarget(
  left: ItineraryComposerTarget | null,
  right: ItineraryComposerTarget,
): boolean {
  return left?.tripId === right.tripId && left.dayNumber === right.dayNumber;
}

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
  const [workspaceMode, setWorkspaceMode] =
    useState<WorkspaceMode>("view");
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
  const [creatingTrip, setCreatingTrip] = useState(false);
  const creatingTripRef = useRef(false);
  const [itineraryComposerTarget, setItineraryComposerTarget] =
    useState<ItineraryComposerTarget | null>(null);
  const itineraryComposerTargetRef = useRef<ItineraryComposerTarget | null>(
    null,
  );
  const itineraryRefreshTargetRef = useRef<ItineraryComposerTarget | null>(
    null,
  );
  const [creatingItinerary, setCreatingItinerary] = useState(false);
  const creatingItineraryRef = useRef(false);
  const [error, setError] = useState<DesktopErrorPayload | null>(null);
  const [restoreWarning, setRestoreWarning] =
    useState<DesktopErrorPayload | null>(null);

  const closeItineraryComposer = useCallback(() => {
    itineraryComposerTargetRef.current = null;
    setItineraryComposerTarget(null);
  }, []);

  const leaveItineraryContext = useCallback(() => {
    closeItineraryComposer();
    itineraryRefreshTargetRef.current = null;
    setLoadingTimeline(false);
  }, [closeItineraryComposer]);

  const clearTripSelection = useCallback(() => {
    leaveItineraryContext();
    setSelectedTripId(null);
    setTripDetail(null);
    setSelectedDayNumber(null);
    setDayTimeline(null);
  }, [leaveItineraryContext]);

  const clearAllData = useCallback(() => {
    setDatabasePath(null);
    setTrips([]);
    clearTripSelection();
    setMainView("trips");
    setWorkspaceMode("view");
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

  const loadTrips = useCallback(async (preferredTripId?: number) => {
    setLoadingTrips(true);
    try {
      const summaries = await api.listTripSummaries();
      setTrips(summaries);
      if (summaries.length > 0) {
        const selectedId =
          summaries.find((trip) => trip.id === preferredTripId)?.id ??
          summaries[0].id;
        setSelectedTripId(selectedId);
        await loadTripDetail(selectedId);
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
    leaveItineraryContext();
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
    leaveItineraryContext,
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
      leaveItineraryContext();
      setMainView("trips");
      setWorkspaceMode("view");
      if (tripId === selectedTripId) {
        return;
      }
      setError(null);
      setSelectedTripId(tripId);
      await loadTripDetail(tripId);
    },
    [leaveItineraryContext, loadTripDetail, selectedTripId],
  );

  const handleCreateTrip = useCallback(
    async (input: CreateTripInput) => {
      if (creatingTripRef.current) {
        return;
      }
      creatingTripRef.current = true;
      setError(null);
      setCreatingTrip(true);
      let createdTripId: number;
      try {
        const result = await api.createTrip(input);
        createdTripId = result.trip_id;
      } catch (err) {
        setError(toDesktopError(err));
        creatingTripRef.current = false;
        setCreatingTrip(false);
        return;
      }

      setWorkspaceMode("view");
      creatingTripRef.current = false;
      setCreatingTrip(false);
      await loadTrips(createdTripId);
    },
    [loadTrips],
  );

  const handleSelectDay = useCallback(
    async (dayNumber: number) => {
      if (
        !selectedTripId ||
        (dayNumber === selectedDayNumber && dayTimeline !== null)
      ) {
        return;
      }
      leaveItineraryContext();
      setError(null);
      if (dayNumber !== selectedDayNumber) {
        setSelectedDayNumber(dayNumber);
      }
      setDayTimeline(null);
      await loadTimeline(selectedTripId, dayNumber);
    },
    [
      dayTimeline,
      leaveItineraryContext,
      loadTimeline,
      selectedDayNumber,
      selectedTripId,
    ],
  );

  const handleOpenItineraryComposer = useCallback(() => {
    if (!selectedTripId || selectedDayNumber === null || loadingTimeline) {
      return;
    }
    const target = {
      tripId: selectedTripId,
      dayNumber: selectedDayNumber,
    };
    itineraryComposerTargetRef.current = target;
    setItineraryComposerTarget(target);
  }, [loadingTimeline, selectedDayNumber, selectedTripId]);

  const handleCreateItinerary = useCallback(
    async (input: CreateItineraryInput) => {
      if (creatingItineraryRef.current) {
        return;
      }
      const target = itineraryComposerTargetRef.current;
      if (
        !target ||
        input.trip_id !== target.tripId ||
        input.day_number !== target.dayNumber
      ) {
        return;
      }

      creatingItineraryRef.current = true;
      setCreatingItinerary(true);
      setError(null);
      try {
        await api.createItinerary(input);
      } catch (err) {
        setError(toDesktopError(err));
        creatingItineraryRef.current = false;
        setCreatingItinerary(false);
        return;
      }

      const targetIsCurrent = sameComposerTarget(
        itineraryComposerTargetRef.current,
        target,
      );
      closeItineraryComposer();
      creatingItineraryRef.current = false;
      setCreatingItinerary(false);
      if (!targetIsCurrent) {
        return;
      }

      itineraryRefreshTargetRef.current = target;
      setLoadingTimeline(true);
      try {
        const timeline = await api.getDayTimeline(
          target.tripId,
          target.dayNumber,
        );
        if (sameComposerTarget(itineraryRefreshTargetRef.current, target)) {
          setDayTimeline(timeline);
        }
      } catch (err) {
        if (sameComposerTarget(itineraryRefreshTargetRef.current, target)) {
          setError(toDesktopError(err));
          setDayTimeline(null);
        }
      } finally {
        if (sameComposerTarget(itineraryRefreshTargetRef.current, target)) {
          itineraryRefreshTargetRef.current = null;
          setLoadingTimeline(false);
        }
      }
    },
    [closeItineraryComposer],
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
            <p className="app-subtitle">Developer preview</p>
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
          <p className="app-subtitle">Developer preview</p>
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
                <div className="sidebar-title-row">
                  <h2>Trips</h2>
                  {!loadingTrips ? (
                    <p className="trip-count" aria-live="polite">
                      {tripCountLabel}
                    </p>
                  ) : null}
                </div>
                <button
                  type="button"
                  className={
                    workspaceMode === "create"
                      ? "new-trip-button selected"
                      : "new-trip-button"
                  }
                  aria-pressed={workspaceMode === "create"}
                  onClick={() => {
                    leaveItineraryContext();
                    setError(null);
                    setWorkspaceMode("create");
                  }}
                >
                  New Trip
                </button>
              </div>
              <TripList
                trips={trips}
                selectedTripId={
                  workspaceMode === "create" ? null : selectedTripId
                }
                loading={loadingTrips}
                onSelect={handleSelectTrip}
              />
            </div>
            <div className="sidebar-footer">
              <button
                type="button"
                className="nav-settings"
                onClick={() => {
                  leaveItineraryContext();
                  setWorkspaceMode("view");
                  setMainView("settings");
                }}
              >
                Settings
              </button>
            </div>
          </aside>

          <main className="detail-pane">
            {workspaceMode === "create" ? (
              <TripCreateForm
                submitting={creatingTrip}
                onSubmit={handleCreateTrip}
                onCancel={() => setWorkspaceMode("view")}
              />
            ) : (
              <TripDetailPanel
                trip={tripDetail}
                selectedDayNumber={selectedDayNumber}
                loading={loadingDetail}
                onSelectDay={handleSelectDay}
                onAddActivity={handleOpenItineraryComposer}
                addActivityDisabled={loadingTimeline || creatingItinerary}
              >
                {itineraryComposerTarget &&
                itineraryComposerTarget.tripId === selectedTripId &&
                itineraryComposerTarget.dayNumber === selectedDayNumber ? (
                  <ItineraryQuickAddForm
                    tripId={itineraryComposerTarget.tripId}
                    dayNumber={itineraryComposerTarget.dayNumber}
                    submitting={creatingItinerary}
                    onSubmit={handleCreateItinerary}
                    onCancel={closeItineraryComposer}
                  />
                ) : null}
                <ItineraryTimeline
                  items={dayTimeline?.itineraries ?? null}
                  loading={loadingTimeline}
                />
              </TripDetailPanel>
            )}
          </main>
        </div>
      )}
    </div>
  );
}
