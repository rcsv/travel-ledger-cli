import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor, within } from "@testing-library/react";

import App from "./App";
import * as api from "./api";

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: vi.fn(),
}));

vi.mock("./api", () => ({
  selectDatabase: vi.fn(),
  listTripSummaries: vi.fn(),
  getTripDetail: vi.fn(),
  getDayTimeline: vi.fn(),
}));

const { open } = await import("@tauri-apps/plugin-dialog");

const sampleTrips = [
  {
    id: 1,
    name: "Okinawa",
    start_date: "2026-04-26",
    end_date: "2026-04-29",
    created_at: "t",
    updated_at: "t",
  },
];

const sampleDetail = {
  id: 1,
  name: "Okinawa",
  start_date: "2026-04-26",
  end_date: "2026-04-29",
  created_at: "t",
  updated_at: "t",
  days: [
    {
      id: 10,
      trip_id: 1,
      day_number: 1,
      date: "2026-04-26",
      title: "",
      summary: null,
    },
    {
      id: 11,
      trip_id: 1,
      day_number: 2,
      date: "2026-04-27",
      title: "",
      summary: null,
    },
  ],
};

beforeEach(() => {
  vi.resetAllMocks();
});

describe("App", () => {
  it("shows database-not-selected empty state initially", () => {
    render(<App />);
    expect(
      screen.getByText("Open a Travel Ledger database"),
    ).toBeInTheDocument();
  });

  it("ignores dialog cancel without showing an error", async () => {
    vi.mocked(open).mockResolvedValue(null);
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: /open travel ledger database/i }));
    await waitFor(() => expect(open).toHaveBeenCalled());
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });

  it("loads trips after database selection", async () => {
    vi.mocked(open).mockResolvedValue("/tmp/sample.db");
    vi.mocked(api.selectDatabase).mockResolvedValue({
      path: "/tmp/sample.db",
      trip_count: 1,
    });
    vi.mocked(api.listTripSummaries).mockResolvedValue(sampleTrips);
    vi.mocked(api.getTripDetail).mockResolvedValue(sampleDetail);
    vi.mocked(api.getDayTimeline).mockResolvedValue({
      trip_id: 1,
      trip_name: "Okinawa",
      day_id: 10,
      day_number: 1,
      date: "2026-04-26",
      title: "",
      summary: null,
      itineraries: [],
    });

    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: /open travel ledger database/i }));

    await waitFor(() =>
      expect(
        within(screen.getByLabelText("Trip list")).getByText("Okinawa"),
      ).toBeInTheDocument(),
    );
    expect(api.selectDatabase).toHaveBeenCalledWith("/tmp/sample.db");
    expect(api.getTripDetail).toHaveBeenCalledWith(1);
  });

  it("shows empty trip state", async () => {
    vi.mocked(open).mockResolvedValue("/tmp/empty.db");
    vi.mocked(api.selectDatabase).mockResolvedValue({
      path: "/tmp/empty.db",
      trip_count: 0,
    });
    vi.mocked(api.listTripSummaries).mockResolvedValue([]);

    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: /open travel ledger database/i }));

    await waitFor(() => expect(screen.getByText("No trips")).toBeInTheDocument());
  });

  it("shows structured error banner", async () => {
    vi.mocked(open).mockResolvedValue("/tmp/bad.db");
    vi.mocked(api.selectDatabase).mockRejectedValue({
      code: "DATABASE_OPEN_FAILED",
      message: "cannot open database",
    });

    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: /open travel ledger database/i }));

    await waitFor(() =>
      expect(screen.getByRole("alert")).toHaveTextContent("DATABASE_OPEN_FAILED"),
    );
  });

  it("updates timeline when day tab is selected", async () => {
    vi.mocked(open).mockResolvedValue("/tmp/sample.db");
    vi.mocked(api.selectDatabase).mockResolvedValue({
      path: "/tmp/sample.db",
      trip_count: 1,
    });
    vi.mocked(api.listTripSummaries).mockResolvedValue(sampleTrips);
    vi.mocked(api.getTripDetail).mockResolvedValue(sampleDetail);
    vi.mocked(api.getDayTimeline)
      .mockResolvedValueOnce({
        trip_id: 1,
        trip_name: "Okinawa",
        day_id: 10,
        day_number: 1,
        date: "2026-04-26",
        title: "",
        summary: null,
        itineraries: [],
      })
      .mockResolvedValueOnce({
        trip_id: 1,
        trip_name: "Okinawa",
        day_id: 11,
        day_number: 2,
        date: "2026-04-27",
        title: "",
        summary: null,
        itineraries: [
          {
            id: 5,
            trip_id: 1,
            day_number: 2,
            title: "Beach",
            sort_order: 1,
            created_at: "t",
            updated_at: "t",
          },
        ],
      });

    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: /open travel ledger database/i }));
    await waitFor(() =>
      expect(
        within(screen.getByLabelText("Trip list")).getByText("Okinawa"),
      ).toBeInTheDocument(),
    );

    fireEvent.click(screen.getByRole("tab", { name: "Day 2" }));
    await waitFor(() => expect(screen.getByText("Beach")).toBeInTheDocument());
    expect(api.getDayTimeline).toHaveBeenLastCalledWith(1, 2);
  });
});
