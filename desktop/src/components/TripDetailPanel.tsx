import type { DaySummary, TripDetail } from "../types";
import { formatDateRange } from "../types";
import { EmptyState } from "./EmptyState";

interface TripDetailPanelProps {
  trip: TripDetail | null;
  selectedDayNumber: number | null;
  loading: boolean;
  onSelectDay: (dayNumber: number) => void;
}

function MetaRow({ label, value }: { label: string; value?: string | null }) {
  if (!value) {
    return null;
  }
  return (
    <div className="meta-row">
      <dt>{label}</dt>
      <dd>{value}</dd>
    </div>
  );
}

function DayTabs({
  days,
  selectedDayNumber,
  onSelectDay,
}: {
  days: DaySummary[];
  selectedDayNumber: number | null;
  onSelectDay: (dayNumber: number) => void;
}) {
  if (days.length === 0) {
    return (
      <div className="empty-state compact">
        <h3>No days</h3>
        <p>This trip has no day rows.</p>
      </div>
    );
  }

  return (
    <div className="day-tabs" role="tablist" aria-label="Days">
      {days.map((day) => {
        const selected = day.day_number === selectedDayNumber;
        return (
          <button
            key={day.id}
            type="button"
            role="tab"
            aria-selected={selected}
            className={selected ? "day-tab selected" : "day-tab"}
            onClick={() => onSelectDay(day.day_number)}
          >
            Day {day.day_number}
          </button>
        );
      })}
    </div>
  );
}

export function TripDetailPanel({
  trip,
  selectedDayNumber,
  loading,
  onSelectDay,
}: TripDetailPanelProps) {
  if (loading) {
    return <p className="status-text">Loading trip detail…</p>;
  }

  if (!trip) {
    return (
      <EmptyState
        title="Select a trip"
        message="Choose a trip from the list to view details and itinerary."
      />
    );
  }

  const selectedDay = trip.days.find((day) => day.day_number === selectedDayNumber);

  return (
    <section className="trip-detail" aria-label="Trip detail">
      <header className="trip-detail-header">
        <h2>{trip.name}</h2>
        <p className="trip-dates">
          {formatDateRange(trip.start_date, trip.end_date)}
        </p>
      </header>

      <dl className="trip-meta-grid">
        <MetaRow label="Summary" value={trip.summary} />
        <MetaRow label="Destination" value={trip.main_destination} />
        <MetaRow label="Country" value={trip.main_destination_country_code} />
        <MetaRow label="Default currency" value={trip.default_currency} />
      </dl>

      <DayTabs
        days={trip.days}
        selectedDayNumber={selectedDayNumber}
        onSelectDay={onSelectDay}
      />

      {selectedDay ? (
        <div className="day-summary">
          <h3>
            Day {selectedDay.day_number} · {selectedDay.date}
          </h3>
          {selectedDay.title ? <p>{selectedDay.title}</p> : null}
          {selectedDay.summary ? <p>{selectedDay.summary}</p> : null}
        </div>
      ) : null}
    </section>
  );
}
