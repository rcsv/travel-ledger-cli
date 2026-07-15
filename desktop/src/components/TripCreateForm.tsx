import { useState, type FormEvent } from "react";

import type { CreateTripInput } from "../types";

interface TripCreateFormProps {
  submitting: boolean;
  onSubmit: (input: CreateTripInput) => void;
  onCancel: () => void;
}

function optionalValue(value: string): string | null {
  return value.trim() === "" ? null : value;
}

export function TripCreateForm({
  submitting,
  onSubmit,
  onCancel,
}: TripCreateFormProps) {
  const [name, setName] = useState("");
  const [startDate, setStartDate] = useState("");
  const [endDate, setEndDate] = useState("");
  const [summary, setSummary] = useState("");
  const [mainDestination, setMainDestination] = useState("");
  const [countryCode, setCountryCode] = useState("");
  const [defaultCurrency, setDefaultCurrency] = useState("");
  const [dateError, setDateError] = useState<string | null>(null);

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (submitting) {
      return;
    }
    if (startDate && endDate && startDate > endDate) {
      setDateError("End date must be on or after start date.");
      return;
    }
    setDateError(null);
    onSubmit({
      name,
      start_date: startDate,
      end_date: endDate,
      summary: optionalValue(summary),
      main_destination: optionalValue(mainDestination),
      main_destination_country_code: optionalValue(countryCode),
      default_currency: optionalValue(defaultCurrency),
    });
  }

  return (
    <section className="trip-create" aria-labelledby="trip-create-heading">
      <header className="trip-create-header">
        <h2 id="trip-create-heading">Create a new trip</h2>
        <p>
          Start with the travel dates and context. Days are created
          automatically.
        </p>
      </header>

      <form className="trip-create-form" onSubmit={handleSubmit}>
        <label className="form-field trip-name-field">
          <span>Trip name</span>
          <input
            type="text"
            name="name"
            required
            autoFocus
            value={name}
            onChange={(event) => setName(event.target.value)}
          />
        </label>

        <div className="form-field-row">
          <label className="form-field">
            <span>Start date</span>
            <input
              type="date"
              name="start_date"
              required
              value={startDate}
              onChange={(event) => setStartDate(event.target.value)}
            />
          </label>
          <label className="form-field">
            <span>End date</span>
            <input
              type="date"
              name="end_date"
              required
              value={endDate}
              onChange={(event) => setEndDate(event.target.value)}
            />
          </label>
        </div>
        {dateError ? (
          <p className="form-validation" role="alert">
            {dateError}
          </p>
        ) : null}

        <label className="form-field">
          <span>Summary</span>
          <textarea
            name="summary"
            rows={4}
            maxLength={2000}
            value={summary}
            onChange={(event) => setSummary(event.target.value)}
          />
        </label>

        <label className="form-field">
          <span>Main destination</span>
          <input
            type="text"
            name="main_destination"
            maxLength={200}
            value={mainDestination}
            onChange={(event) => setMainDestination(event.target.value)}
          />
        </label>

        <div className="form-field-row">
          <label className="form-field">
            <span>Main destination country code</span>
            <input
              type="text"
              name="main_destination_country_code"
              maxLength={2}
              placeholder="JP"
              value={countryCode}
              onChange={(event) => setCountryCode(event.target.value)}
            />
          </label>
          <label className="form-field">
            <span>Default currency</span>
            <input
              type="text"
              name="default_currency"
              maxLength={3}
              placeholder="JPY"
              value={defaultCurrency}
              onChange={(event) => setDefaultCurrency(event.target.value)}
            />
          </label>
        </div>

        <div className="trip-create-actions">
          <button type="submit" className="primary-button" disabled={submitting}>
            {submitting ? "Creating…" : "Create trip"}
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
