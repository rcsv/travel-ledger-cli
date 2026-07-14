import type { TripSummary } from "../types";
import { formatDateRange, nonEmpty } from "../display";

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
        <h3>No trips yet</h3>
        <p>
          This database does not contain any trips. Open another database, or
          add trips with the CLI.
        </p>
      </div>
    );
  }

  return (
    <ul className="trip-list" aria-label="Trip list">
      {trips.map((trip) => {
        const selected = trip.id === selectedTripId;
        const dates = formatDateRange(trip.start_date, trip.end_date);
        const destination = nonEmpty(trip.main_destination);
        const currency = nonEmpty(trip.default_currency);
        return (
          <li key={trip.id}>
            <button
              type="button"
              className={selected ? "trip-item selected" : "trip-item"}
              aria-current={selected ? "true" : undefined}
              aria-pressed={selected}
              onClick={() => onSelect(trip.id)}
            >
              <span className="trip-name">{trip.name}</span>
              {dates ? <span className="trip-meta">{dates}</span> : null}
              {destination ? (
                <span className="trip-meta">{destination}</span>
              ) : null}
              {currency ? (
                <span className="trip-meta trip-currency">{currency}</span>
              ) : null}
            </button>
          </li>
        );
      })}
    </ul>
  );
}
