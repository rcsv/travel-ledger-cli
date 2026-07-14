import type { ItineraryDetail } from "../types";
import { formatMinutes, nonEmpty } from "../display";

interface ItineraryTimelineProps {
  items: ItineraryDetail[];
  loading: boolean;
  dayNumber: number | null;
  dayLabel?: string | null;
}

export function ItineraryTimeline({
  items,
  loading,
  dayNumber,
  dayLabel,
}: ItineraryTimelineProps) {
  if (dayNumber === null) {
    return null;
  }

  const heading = dayLabel
    ? `Day ${dayNumber} activities · ${dayLabel}`
    : `Day ${dayNumber} activities`;

  if (loading) {
    return <p className="status-text">Loading activities…</p>;
  }

  if (items.length === 0) {
    return (
      <section className="timeline" aria-label="Day activities">
        <h3>{heading}</h3>
        <div className="empty-state compact">
          <p>No activities planned for this day yet.</p>
        </div>
      </section>
    );
  }

  return (
    <section className="timeline" aria-label="Day activities">
      <h3>{heading}</h3>
      <ol className="timeline-list">
        {items.map((item, index) => {
          const startTime = nonEmpty(item.start_time);
          const category = nonEmpty(item.category);
          const location = nonEmpty(item.location);
          const duration = formatMinutes(item.duration_minutes);
          const travel = formatMinutes(item.travel_minutes);
          const note = nonEmpty(item.note);
          return (
            <li key={item.id} className="timeline-item">
              <div className="timeline-head">
                <span className="timeline-order" aria-hidden="true">
                  {index + 1}
                </span>
                <div className="timeline-title-block">
                  <strong className="timeline-title">{item.title}</strong>
                  {startTime ? (
                    <span className="timeline-time">{startTime}</span>
                  ) : null}
                </div>
              </div>
              {(category || location || duration || travel || note) && (
                <ul className="timeline-fields">
                  {category ? <li>{category}</li> : null}
                  {location ? <li>{location}</li> : null}
                  {duration ? <li>{duration}</li> : null}
                  {travel ? <li>Travel {travel}</li> : null}
                  {note ? <li className="timeline-note">{note}</li> : null}
                </ul>
              )}
            </li>
          );
        })}
      </ol>
      <p className="timeline-footnote">
        Activities follow plan order (sequence first). Times are optional
        labels.
      </p>
    </section>
  );
}
