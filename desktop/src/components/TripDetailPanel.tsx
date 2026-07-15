import type { ReactNode } from "react";

import type { DaySummary, TripDetail } from "../types";
import { formatDateRange, formatDayLabel, nonEmpty } from "../display";
import { EmptyState } from "./EmptyState";

interface TripDetailPanelProps {
  trip: TripDetail | null;
  selectedDayNumber: number | null;
  loading: boolean;
  onSelectDay: (dayNumber: number) => void;
  children?: ReactNode;
}

function ContextFact({
  label,
  value,
}: {
  label: string;
  value?: string | null;
}) {
  const text = nonEmpty(value);
  if (!text) {
    return null;
  }
  return (
    <div className="trip-context-fact">
      <dt>{label}</dt>
      <dd>{text}</dd>
    </div>
  );
}

function DaySelector({
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
      <div className="empty-state compact plan-empty-state">
        <h4>No days yet</h4>
        <p>This trip has no day rows to browse.</p>
      </div>
    );
  }

  return (
    <div className="day-selector" role="group" aria-label="Days">
      {days.map((day) => {
        const selected = day.day_number === selectedDayNumber;
        const label = `Day ${day.day_number} · ${formatDayLabel(day.date)}`;
        return (
          <button
            key={day.id}
            type="button"
            aria-pressed={selected}
            className={selected ? "day-button selected" : "day-button"}
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
  children,
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
  const summary = nonEmpty(trip.summary);
  const destination = nonEmpty(trip.main_destination);
  const country = nonEmpty(trip.main_destination_country_code);
  const currency = nonEmpty(trip.default_currency);

  return (
    <section
      className="trip-workspace"
      aria-labelledby="trip-workspace-heading"
    >
      <header className="trip-context-header">
        <h2 id="trip-workspace-heading">{trip.name}</h2>
        <p className="trip-context-dates">
          {dates ? dates : null}
          {dates ? " · " : null}
          {daysLabel}
        </p>

        {destination || country || currency ? (
          <dl className="trip-context-facts" aria-label="Trip details">
            <ContextFact label="Destination" value={destination} />
            <ContextFact label="Country" value={country} />
            <ContextFact label="Currency" value={currency} />
          </dl>
        ) : null}

        {summary ? (
          <p className="trip-context-summary" aria-label="Trip summary">
            {summary}
          </p>
        ) : null}
      </header>

      <section className="plan-section" aria-labelledby="plan-heading">
        <h3 id="plan-heading">Plan</h3>
        <DaySelector
          days={trip.days}
          selectedDayNumber={selectedDayNumber}
          onSelectDay={onSelectDay}
        />

        {selectedDay ? (
          <section
            className="selected-day-plan"
            aria-labelledby="selected-day-heading"
          >
            <header className="selected-day-header">
              <h4 id="selected-day-heading">
                Day {selectedDay.day_number} · {formatDayLabel(selectedDay.date)}
              </h4>
              {nonEmpty(selectedDay.title) ? (
                <p className="selected-day-title">{selectedDay.title}</p>
              ) : null}
              {nonEmpty(selectedDay.summary) ? (
                <p className="selected-day-summary">{selectedDay.summary}</p>
              ) : null}
            </header>
            {children}
          </section>
        ) : null}
      </section>
    </section>
  );
}
