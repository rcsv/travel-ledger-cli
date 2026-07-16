import { useEffect, useRef, useState, type FormEvent } from "react";

import type { ItineraryDetail, UpdateItineraryInput } from "../types";

interface ActivityInspectorFormProps {
  item: ItineraryDetail;
  submitting: boolean;
  onSubmit: (input: UpdateItineraryInput) => void;
  onCancel: () => void;
}

function optionalValue(value: string): string | null {
  return value.trim() === "" ? null : value;
}

export function ActivityInspectorForm({
  item,
  submitting,
  onSubmit,
  onCancel,
}: ActivityInspectorFormProps) {
  const [title, setTitle] = useState(item.title);
  const [startTime, setStartTime] = useState(item.start_time ?? "");
  const [location, setLocation] = useState(item.location ?? "");
  const [note, setNote] = useState(item.note ?? "");
  const submittingRef = useRef(false);
  const unchanged =
    title === item.title &&
    startTime === (item.start_time ?? "") &&
    location === (item.location ?? "") &&
    note === (item.note ?? "");

  useEffect(() => {
    if (!submitting) {
      submittingRef.current = false;
    }
  }, [submitting]);

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (submitting || submittingRef.current || unchanged) {
      return;
    }
    submittingRef.current = true;
    onSubmit({
      trip_id: item.trip_id,
      day_number: item.day_number,
      itinerary_id: item.id,
      title,
      start_time: optionalValue(startTime),
      location: optionalValue(location),
      note: optionalValue(note),
    });
  }

  return (
    <section
      id="activity-inspector"
      className="activity-inspector"
      aria-labelledby="activity-inspector-heading"
    >
      <div className="activity-inspector-heading-row">
        <div>
          <p className="activity-inspector-eyebrow">Selected activity</p>
          <h5 id="activity-inspector-heading">Edit activity</h5>
        </div>
      </div>
      <form className="activity-inspector-form" onSubmit={handleSubmit}>
        <label className="form-field">
          <span>Title</span>
          <input
            type="text"
            name="title"
            required
            autoFocus
            value={title}
            onChange={(event) => setTitle(event.target.value)}
          />
        </label>

        <label className="form-field">
          <span>Start time</span>
          <input
            type="time"
            name="start_time"
            value={startTime}
            onChange={(event) => setStartTime(event.target.value)}
          />
        </label>

        <label className="form-field">
          <span>Location</span>
          <input
            type="text"
            name="location"
            value={location}
            onChange={(event) => setLocation(event.target.value)}
          />
        </label>

        <label className="form-field">
          <span>Note</span>
          <textarea
            name="note"
            rows={5}
            value={note}
            onChange={(event) => setNote(event.target.value)}
          />
        </label>

        <div className="activity-inspector-actions">
          <button
            type="submit"
            className="primary-button"
            disabled={submitting || unchanged}
          >
            {submitting ? "Saving…" : "Save"}
          </button>
          <button
            type="button"
            className="secondary-button"
            disabled={submitting}
            onClick={onCancel}
          >
            Cancel
          </button>
        </div>
      </form>
    </section>
  );
}
