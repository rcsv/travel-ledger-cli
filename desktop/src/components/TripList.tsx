import type { TripSummary } from "../types";
import { formatDateRange } from "../types";

interface TripListProps {
  trips: TripSummary[];
  selectedTripId: number | null;
  loading: boolean;
  onSelect: (tripId: number) => void;
}

export function TripList({
  trips,
  selectedTripId,
  loading,
  onSelect,
}: TripListProps) {
  if (loading) {
    return <p className="status-text">Loading trips…</p>;
  }

  if (trips.length === 0) {
    return (
      <div className="empty-state compact">
        <h3>No trips</h3>
        <p>This database has no trips yet.</p>
      </div>
    );
  }

  return (
    <ul className="trip-list" aria-label="Trip list">
      {trips.map((trip) => {
        const selected = trip.id === selectedTripId;
        return (
          <li key={trip.id}>
            <button
              type="button"
              className={selected ? "trip-item selected" : "trip-item"}
              aria-pressed={selected}
              onClick={() => onSelect(trip.id)}
            >
              <span className="trip-name">{trip.name}</span>
              <span className="trip-meta">
                {formatDateRange(trip.start_date, trip.end_date)}
              </span>
              {trip.main_destination ? (
                <span className="trip-meta">{trip.main_destination}</span>
              ) : null}
            </button>
          </li>
        );
      })}
    </ul>
  );
}
