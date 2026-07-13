import type { ItineraryDetail } from "../types";

interface ItineraryTimelineProps {
  items: ItineraryDetail[];
  loading: boolean;
  dayNumber: number | null;
}

function formatMinutes(value?: number | null): string | null {
  if (value === null || value === undefined) {
    return null;
  }
  return `${value} min`;
}

export function ItineraryTimeline({
  items,
  loading,
  dayNumber,
}: ItineraryTimelineProps) {
  if (dayNumber === null) {
    return null;
  }

  if (loading) {
    return <p className="status-text">Loading itinerary…</p>;
  }

  if (items.length === 0) {
    return (
      <section className="timeline" aria-label="Itinerary timeline">
        <h3>Itinerary timeline</h3>
        <div className="empty-state compact">
          <p>No itinerary items for Day {dayNumber}.</p>
        </div>
      </section>
    );
  }

  return (
    <section className="timeline" aria-label="Itinerary timeline">
      <h3>Itinerary timeline</h3>
      <ol className="timeline-list">
        {items.map((item) => (
          <li key={item.id} className="timeline-item">
            <div className="timeline-head">
              <span className="timeline-order">#{item.sort_order}</span>
              <strong>{item.title}</strong>
            </div>
            <ul className="timeline-fields">
              {item.start_time ? <li>Start: {item.start_time}</li> : null}
              {item.category ? <li>Category: {item.category}</li> : null}
              {item.location ? <li>Location: {item.location}</li> : null}
              {formatMinutes(item.duration_minutes) ? (
                <li>Duration: {formatMinutes(item.duration_minutes)}</li>
              ) : null}
              {formatMinutes(item.travel_minutes) ? (
                <li>Travel: {formatMinutes(item.travel_minutes)}</li>
              ) : null}
              {item.note ? <li>Note: {item.note}</li> : null}
            </ul>
          </li>
        ))}
      </ol>
    </section>
  );
}
