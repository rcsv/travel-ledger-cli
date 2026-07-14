import type { DaySummary, TripDetail } from "../types";
import { formatDateRange, formatDayLabel, nonEmpty } from "../display";
import { EmptyState } from "./EmptyState";

interface TripDetailPanelProps {
  trip: TripDetail | null;
  selectedDayNumber: number | null;
  loading: boolean;
  onSelectDay: (dayNumber: number) => void;
}

function MetaRow({ label, value }: { label: string; value?: string | null }) {
  const text = nonEmpty(value);
  if (!text) {
    return null;
  }
  return (
    <div className="meta-row">
      <dt>{label}</dt>
      <dd>{text}</dd>
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
        <h3>No days yet</h3>
        <p>This trip has no day rows to browse.</p>
      </div>
    );
  }

  return (
    <div className="day-tabs" role="tablist" aria-label="Days">
      {days.map((day) => {
        const selected = day.day_number === selectedDayNumber;
        const label = `Day ${day.day_number} · ${formatDayLabel(day.date)}`;
        return (
          <button
            key={day.id}
            type="button"
            role="tab"
            aria-selected={selected}
            className={selected ? "day-tab selected" : "day-tab"}
            onClick={() => onSelectDay(day.day_number)}
          >
            {label}
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
    return <p className="status-text">Loading trip…</p>;
  }

  if (!trip) {
    return (
      <EmptyState
        title="Pick a trip"
        message="Choose a trip on the left to see its days and itinerary."
      />
    );
  }

  const selectedDay = trip.days.find(
    (day) => day.day_number === selectedDayNumber,
  );
  const dates = formatDateRange(trip.start_date, trip.end_date);
  const daysLabel =
    trip.days.length === 1 ? "1 day" : `${trip.days.length} days`;

  return (
    <section className="trip-detail" aria-label="Trip detail">
      <header className="trip-detail-header">
        <h2>{trip.name}</h2>
        <p className="trip-dates">
          {dates ? dates : null}
          {dates ? " · " : null}
          {daysLabel}
        </p>
      </header>

      <dl className="trip-meta-grid">
        <MetaRow label="Summary" value={trip.summary} />
        <MetaRow label="Destination" value={trip.main_destination} />
        <MetaRow label="Country" value={trip.main_destination_country_code} />
        <MetaRow label="Currency" value={trip.default_currency} />
      </dl>

      <h3 className="section-label">Days</h3>
      <DayTabs
        days={trip.days}
        selectedDayNumber={selectedDayNumber}
        onSelectDay={onSelectDay}
      />

      {selectedDay ? (
        <div className="day-summary">
          <h3>
            Day {selectedDay.day_number} · {formatDayLabel(selectedDay.date)}
          </h3>
          {nonEmpty(selectedDay.title) ? <p>{selectedDay.title}</p> : null}
          {nonEmpty(selectedDay.summary) ? <p>{selectedDay.summary}</p> : null}
        </div>
      ) : null}
    </section>
  );
}
