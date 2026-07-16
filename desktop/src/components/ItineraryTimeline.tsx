import type { ItineraryDetail } from "../types";
import { formatMinutes, nonEmpty } from "../display";

interface ItineraryTimelineProps {
  items: ItineraryDetail[] | null;
  loading: boolean;
  selectedItineraryId: number | null;
  editingDisabled: boolean;
  onEdit: (item: ItineraryDetail) => void;
}

export function ItineraryTimeline({
  items,
  loading,
  selectedItineraryId,
  editingDisabled,
  onEdit,
}: ItineraryTimelineProps) {
  if (loading) {
    return (
      <p className="status-text timeline-status" role="status">
        Loading activities…
      </p>
    );
  }

  if (items === null) {
    return null;
  }

  if (items.length === 0) {
    return (
      <section className="timeline" aria-label="Itinerary timeline">
        <div className="empty-state compact">
          <p>No activities planned for this day yet.</p>
        </div>
      </section>
    );
  }

  return (
    <section className="timeline" aria-label="Itinerary timeline">
      <ol className="timeline-list">
        {items.map((item, index) => {
          const selected = item.id === selectedItineraryId;
          const startTime = nonEmpty(item.start_time);
          const category = nonEmpty(item.category);
          const location = nonEmpty(item.location);
          const duration = formatMinutes(item.duration_minutes);
          const travel = formatMinutes(item.travel_minutes);
          const note = nonEmpty(item.note);
          return (
            <li
              key={item.id}
              className={selected ? "timeline-item selected" : "timeline-item"}
            >
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
                <button
                  id={`activity-edit-${item.id}`}
                  type="button"
                  className="timeline-edit-button"
                  aria-label={`Edit activity: ${item.title}`}
                  aria-expanded={selected}
                  aria-controls={selected ? "activity-inspector" : undefined}
                  disabled={editingDisabled}
                  onClick={() => onEdit(item)}
                >
                  Edit
                </button>
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
