import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor, within } from "@testing-library/react";

import App from "./App";
import * as api from "./api";

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: vi.fn(),
}));

vi.mock("./api", () => ({
  selectDatabase: vi.fn(),
  restoreLastDatabase: vi.fn(),
  forgetDatabase: vi.fn(),
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

const emptyTimeline = {
  trip_id: 1,
  trip_name: "Okinawa",
  day_id: 10,
  day_number: 1,
  date: "2026-04-26",
  title: "",
  summary: null,
  itineraries: [],
};

async function finishBootstrap() {
  await waitFor(() =>
    expect(screen.queryByText("Starting…")).not.toBeInTheDocument(),
  );
}

beforeEach(() => {
  vi.resetAllMocks();
  vi.mocked(api.restoreLastDatabase).mockResolvedValue({ status: "not_found" });
});

describe("App", () => {
  it("shows starting state while restoring", async () => {
    let resolveRestore: (value: { status: "not_found" }) => void = () => {};
    vi.mocked(api.restoreLastDatabase).mockReturnValue(
      new Promise((resolve) => {
        resolveRestore = resolve;
      }),
    );
    render(<App />);
    expect(screen.getByText("Starting…")).toBeInTheDocument();
    resolveRestore({ status: "not_found" });
    await finishBootstrap();
  });

  it("shows database-not-selected empty state when nothing saved", async () => {
    render(<App />);
    await finishBootstrap();
    expect(
      screen.getByText("Open a Travel Ledger database"),
    ).toBeInTheDocument();
  });

  it("loads trips after successful restore", async () => {
    vi.mocked(api.restoreLastDatabase).mockResolvedValue({
      status: "restored",
      database: { path: "/tmp/sample.db", trip_count: 1 },
    });
    vi.mocked(api.listTripSummaries).mockResolvedValue(sampleTrips);
    vi.mocked(api.getTripDetail).mockResolvedValue(sampleDetail);
    vi.mocked(api.getDayTimeline).mockResolvedValue(emptyTimeline);

    render(<App />);
    await waitFor(() =>
      expect(
        within(screen.getByLabelText("Trip list")).getByText("Okinawa"),
      ).toBeInTheDocument(),
    );
    expect(screen.getByText("sample.db")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /change database/i })).toBeInTheDocument();
  });

  it("shows restore warning when saved DB is invalid", async () => {
    vi.mocked(api.restoreLastDatabase).mockResolvedValue({
      status: "invalid_cleared",
      code: "DATABASE_PATH_INVALID",
      message: "Database file does not exist",
    });
    render(<App />);
    await finishBootstrap();
    expect(screen.getByRole("alert")).toHaveTextContent("DATABASE_PATH_INVALID");
    expect(
      screen.getByText("Open a Travel Ledger database"),
    ).toBeInTheDocument();
  });

  it("ignores dialog cancel without showing an error", async () => {
    vi.mocked(open).mockResolvedValue(null);
    render(<App />);
    await finishBootstrap();
    fireEvent.click(
      screen.getByRole("button", { name: /open travel ledger database/i }),
    );
    await waitFor(() => expect(open).toHaveBeenCalled());
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });

  it("keeps current database when change fails", async () => {
    vi.mocked(api.restoreLastDatabase).mockResolvedValue({
      status: "restored",
      database: { path: "/tmp/sample.db", trip_count: 1 },
    });
    vi.mocked(api.listTripSummaries).mockResolvedValue(sampleTrips);
    vi.mocked(api.getTripDetail).mockResolvedValue(sampleDetail);
    vi.mocked(api.getDayTimeline).mockResolvedValue(emptyTimeline);
    vi.mocked(open).mockResolvedValue("/tmp/bad.db");
    vi.mocked(api.selectDatabase).mockRejectedValue({
      code: "DATABASE_OPEN_FAILED",
      message: "cannot open",
    });

    render(<App />);
    await waitFor(() => expect(screen.getByText("sample.db")).toBeInTheDocument());

    fireEvent.click(screen.getByRole("button", { name: /change database/i }));
    await waitFor(() =>
      expect(screen.getByRole("alert")).toHaveTextContent("DATABASE_OPEN_FAILED"),
    );
    expect(screen.getByText("sample.db")).toBeInTheDocument();
    expect(
      within(screen.getByLabelText("Trip list")).getByText("Okinawa"),
    ).toBeInTheDocument();
  });

  it("changes database and clears previous trip selection", async () => {
    vi.mocked(api.restoreLastDatabase).mockResolvedValue({
      status: "restored",
      database: { path: "/tmp/sample.db", trip_count: 1 },
    });
    vi.mocked(api.listTripSummaries)
      .mockResolvedValueOnce(sampleTrips)
      .mockResolvedValueOnce([
        {
          id: 2,
          name: "Hawaii",
          start_date: "2026-07-01",
          end_date: "2026-07-05",
          created_at: "t",
          updated_at: "t",
        },
      ]);
    vi.mocked(api.getTripDetail)
      .mockResolvedValueOnce(sampleDetail)
      .mockResolvedValueOnce({
        ...sampleDetail,
        id: 2,
        name: "Hawaii",
        days: [
          {
            id: 20,
            trip_id: 2,
            day_number: 1,
            date: "2026-07-01",
            title: "",
            summary: null,
          },
        ],
      });
    vi.mocked(api.getDayTimeline).mockResolvedValue(emptyTimeline);
    vi.mocked(open).mockResolvedValue("/tmp/hawaii.db");
    vi.mocked(api.selectDatabase).mockResolvedValue({
      path: "/tmp/hawaii.db",
      trip_count: 1,
    });

    render(<App />);
    await waitFor(() => expect(screen.getByText("sample.db")).toBeInTheDocument());

    fireEvent.click(screen.getByRole("button", { name: /change database/i }));
    await waitFor(() => expect(screen.getByText("hawaii.db")).toBeInTheDocument());
    expect(
      within(screen.getByLabelText("Trip list")).getByText("Hawaii"),
    ).toBeInTheDocument();
  });

  it("forgets database after confirmation", async () => {
    vi.mocked(api.restoreLastDatabase).mockResolvedValue({
      status: "restored",
      database: { path: "/tmp/sample.db", trip_count: 1 },
    });
    vi.mocked(api.listTripSummaries).mockResolvedValue(sampleTrips);
    vi.mocked(api.getTripDetail).mockResolvedValue(sampleDetail);
    vi.mocked(api.getDayTimeline).mockResolvedValue(emptyTimeline);
    vi.mocked(api.forgetDatabase).mockResolvedValue(undefined);
    const confirmSpy = vi.spyOn(window, "confirm").mockReturnValue(true);

    render(<App />);
    await waitFor(() => expect(screen.getByText("sample.db")).toBeInTheDocument());

    fireEvent.click(screen.getByRole("button", { name: /forget database/i }));
    await waitFor(() =>
      expect(
        screen.getByText("Open a Travel Ledger database"),
      ).toBeInTheDocument(),
    );
    expect(api.forgetDatabase).toHaveBeenCalled();
    expect(confirmSpy).toHaveBeenCalled();
    confirmSpy.mockRestore();
  });

  it("shows empty trip state", async () => {
    vi.mocked(open).mockResolvedValue("/tmp/empty.db");
    vi.mocked(api.selectDatabase).mockResolvedValue({
      path: "/tmp/empty.db",
      trip_count: 0,
    });
    vi.mocked(api.listTripSummaries).mockResolvedValue([]);

    render(<App />);
    await finishBootstrap();
    fireEvent.click(
      screen.getByRole("button", { name: /open travel ledger database/i }),
    );
    await waitFor(() => expect(screen.getByText("No trips")).toBeInTheDocument());
  });

  it("updates timeline when day tab is selected", async () => {
    vi.mocked(api.restoreLastDatabase).mockResolvedValue({
      status: "restored",
      database: { path: "/tmp/sample.db", trip_count: 1 },
    });
    vi.mocked(api.listTripSummaries).mockResolvedValue(sampleTrips);
    vi.mocked(api.getTripDetail).mockResolvedValue(sampleDetail);
    vi.mocked(api.getDayTimeline)
      .mockResolvedValueOnce(emptyTimeline)
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
