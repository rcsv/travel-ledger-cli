import { useState, type FormEvent } from "react";

import type { CreateItineraryInput } from "../types";

interface ItineraryQuickAddFormProps {
  tripId: number;
  dayNumber: number;
  submitting: boolean;
  onSubmit: (input: CreateItineraryInput) => void;
  onCancel: () => void;
}

function optionalValue(value: string): string | null {
  return value.trim() === "" ? null : value;
}

export function ItineraryQuickAddForm({
  tripId,
  dayNumber,
  submitting,
  onSubmit,
  onCancel,
}: ItineraryQuickAddFormProps) {
  const [title, setTitle] = useState("");
  const [startTime, setStartTime] = useState("");
  const [location, setLocation] = useState("");
  const [note, setNote] = useState("");

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (submitting) {
      return;
    }
    onSubmit({
      trip_id: tripId,
      day_number: dayNumber,
      title,
      start_time: optionalValue(startTime),
      location: optionalValue(location),
      note: optionalValue(note),
    });
  }

  return (
    <section
      className="itinerary-quick-add"
      aria-labelledby="itinerary-quick-add-heading"
    >
      <h5 id="itinerary-quick-add-heading">Add activity</h5>
      <form className="itinerary-quick-add-form" onSubmit={handleSubmit}>
        <label className="form-field itinerary-title-field">
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

        <div className="form-field-row">
          <label className="form-field itinerary-time-field">
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
        </div>

        <label className="form-field">
          <span>Note</span>
          <textarea
            name="note"
            rows={3}
            value={note}
            onChange={(event) => setNote(event.target.value)}
          />
        </label>

        <div className="itinerary-quick-add-actions">
          <button type="submit" className="primary-button" disabled={submitting}>
            {submitting ? "Adding…" : "Add activity"}
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
