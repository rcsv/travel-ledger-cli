import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor, within } from "@testing-library/react";

import App from "./App";
import * as api from "./api";

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: vi.fn(),
}));

vi.mock("@tauri-apps/api/app", () => ({
  getVersion: vi.fn().mockResolvedValue("4.10.3"),
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
const { getVersion } = await import("@tauri-apps/api/app");

const sampleTrips = [
  {
    id: 1,
    name: "Okinawa",
    start_date: "2026-04-26",
    end_date: "2026-04-29",
    main_destination: "Naha",
    default_currency: "JPY",
    created_at: "t",
    updated_at: "t",
  },
];

const sampleDetail = {
  id: 1,
  name: "Okinawa",
  start_date: "2026-04-26",
  end_date: "2026-04-29",
  main_destination: "Naha",
  default_currency: "JPY",
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

async function restoreWithSampleTrip() {
  vi.mocked(api.restoreLastDatabase).mockResolvedValue({
    status: "restored",
    database: { path: "/tmp/sample.db", trip_count: 1 },
  });
  vi.mocked(api.listTripSummaries).mockResolvedValue(sampleTrips);
  vi.mocked(api.getTripDetail).mockResolvedValue(sampleDetail);
  vi.mocked(api.getDayTimeline).mockResolvedValue(emptyTimeline);
}

beforeEach(() => {
  vi.resetAllMocks();
  vi.mocked(api.restoreLastDatabase).mockResolvedValue({ status: "not_found" });
  vi.mocked(getVersion).mockResolvedValue("4.10.3");
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

  it("shows database-not-selected empty state with Open Database", async () => {
    render(<App />);
    await finishBootstrap();
    expect(
      screen.getByText("Open a Travel Ledger database"),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /^open database$/i }),
    ).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /^settings$/i })).not.toBeInTheDocument();
  });

  it("loads trips after successful restore and exposes Settings", async () => {
    await restoreWithSampleTrip();
    render(<App />);
    await waitFor(() =>
      expect(
        within(screen.getByLabelText("Trip list")).getByText("Okinawa"),
      ).toBeInTheDocument(),
    );
    expect(screen.getByText("sample.db")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /^settings$/i })).toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: /change database/i }),
    ).not.toBeInTheDocument();
  });

  it("opens Settings with Database and About details", async () => {
    await restoreWithSampleTrip();
    render(<App />);
    await waitFor(() =>
      expect(screen.getByRole("button", { name: /^settings$/i })).toBeInTheDocument(),
    );

    fireEvent.click(screen.getByRole("button", { name: /^settings$/i }));
    const settings = await screen.findByRole("region", { name: "Settings" });
    expect(within(settings).getByRole("heading", { name: "Database" })).toBeInTheDocument();
    expect(within(settings).getByRole("heading", { name: "About" })).toBeInTheDocument();
    expect(within(settings).getByText("sample.db")).toBeInTheDocument();
    expect(within(settings).getByText("/tmp/sample.db")).toBeInTheDocument();
    expect(within(settings).getByText("Access: read-only")).toBeInTheDocument();
    expect(within(settings).getByText("Travel Ledger Desktop")).toBeInTheDocument();
    await waitFor(() =>
      expect(within(settings).getByText("4.10.3")).toBeInTheDocument(),
    );
    expect(
      within(settings).getByText(/SQLite database file is not deleted/i),
    ).toBeInTheDocument();
  });

  it("preserves trip selection when opening and leaving Settings", async () => {
    await restoreWithSampleTrip();
    render(<App />);
    await waitFor(() =>
      expect(
        within(screen.getByLabelText("Trip list")).getByText("Okinawa"),
      ).toBeInTheDocument(),
    );

    fireEvent.click(screen.getByRole("button", { name: /^settings$/i }));
    expect(screen.getByRole("region", { name: "Settings" })).toBeInTheDocument();
    expect(
      within(screen.getByLabelText("Trip list")).getByText("Okinawa"),
    ).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: /back to trips/i }));
    await waitFor(() =>
      expect(screen.getByLabelText("Trip detail")).toHaveTextContent("Okinawa"),
    );
    expect(api.getTripDetail).toHaveBeenCalledTimes(1);
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
    fireEvent.click(screen.getByRole("button", { name: /^open database$/i }));
    await waitFor(() => expect(open).toHaveBeenCalled());
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });

  it("keeps current database when Change fails from Settings", async () => {
    await restoreWithSampleTrip();
    vi.mocked(open).mockResolvedValue("/tmp/bad.db");
    vi.mocked(api.selectDatabase).mockRejectedValue({
      code: "DATABASE_OPEN_FAILED",
      message: "cannot open",
    });

    render(<App />);
    await waitFor(() => expect(screen.getByText("sample.db")).toBeInTheDocument());

    fireEvent.click(screen.getByRole("button", { name: /^settings$/i }));
    fireEvent.click(screen.getByRole("button", { name: /change database/i }));
    await waitFor(() =>
      expect(screen.getByRole("alert")).toHaveTextContent("DATABASE_OPEN_FAILED"),
    );
    expect(screen.getAllByText("sample.db").length).toBeGreaterThan(0);
    expect(
      within(screen.getByLabelText("Trip list")).getByText("Okinawa"),
    ).toBeInTheDocument();
    expect(screen.getByRole("region", { name: "Settings" })).toBeInTheDocument();
  });

  it("changes database from Settings and keeps Settings reachable", async () => {
    await restoreWithSampleTrip();
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
        main_destination: null,
        default_currency: null,
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
    vi.mocked(open).mockResolvedValue("/tmp/hawaii.db");
    vi.mocked(api.selectDatabase).mockResolvedValue({
      path: "/tmp/hawaii.db",
      trip_count: 1,
    });

    render(<App />);
    await waitFor(() => expect(screen.getByText("sample.db")).toBeInTheDocument());

    fireEvent.click(screen.getByRole("button", { name: /^settings$/i }));
    fireEvent.click(screen.getByRole("button", { name: /change database/i }));
    await waitFor(() =>
      expect(screen.getAllByText("hawaii.db").length).toBeGreaterThan(0),
    );
    expect(
      within(screen.getByLabelText("Trip list")).getByText("Hawaii"),
    ).toBeInTheDocument();
    expect(screen.getByRole("region", { name: "Settings" })).toBeInTheDocument();
  });

  it("forgets database from Settings after confirmation", async () => {
    await restoreWithSampleTrip();
    vi.mocked(api.forgetDatabase).mockResolvedValue(undefined);
    const confirmSpy = vi.spyOn(window, "confirm").mockReturnValue(true);

    render(<App />);
    await waitFor(() => expect(screen.getByText("sample.db")).toBeInTheDocument());

    fireEvent.click(screen.getByRole("button", { name: /^settings$/i }));
    fireEvent.click(screen.getByRole("button", { name: /forget database/i }));
    await waitFor(() =>
      expect(
        screen.getByText("Open a Travel Ledger database"),
      ).toBeInTheDocument(),
    );
    expect(api.forgetDatabase).toHaveBeenCalled();
    expect(confirmSpy.mock.calls[0]?.[0]).toMatch(/stays on disk/i);
    expect(
      screen.getByRole("button", { name: /^open database$/i }),
    ).toBeInTheDocument();
    confirmSpy.mockRestore();
  });

  it("shows empty trip state and Settings", async () => {
    vi.mocked(open).mockResolvedValue("/tmp/empty.db");
    vi.mocked(api.selectDatabase).mockResolvedValue({
      path: "/tmp/empty.db",
      trip_count: 0,
    });
    vi.mocked(api.listTripSummaries).mockResolvedValue([]);

    render(<App />);
    await finishBootstrap();
    fireEvent.click(screen.getByRole("button", { name: /^open database$/i }));
    await waitFor(() =>
      expect(screen.getByText("No trips yet")).toBeInTheDocument(),
    );
    expect(screen.getByRole("button", { name: /^settings$/i })).toBeInTheDocument();
  });

  it("updates timeline when day tab is selected", async () => {
    await restoreWithSampleTrip();
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
            start_time: "10:00",
            sort_order: 1,
            location: "Sesoko",
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

    fireEvent.click(screen.getByRole("tab", { name: /Day 2 · Mon · Apr 27/i }));
    await waitFor(() => expect(screen.getByText("Beach")).toBeInTheDocument());
    expect(api.getDayTimeline).toHaveBeenLastCalledWith(1, 2);
  });

  it("wraps long database paths in Settings", async () => {
    const longPath =
      "/Users/example/very/long/nested/directories/travel-ledger/databases/okinawa-sample-file.db";
    vi.mocked(api.restoreLastDatabase).mockResolvedValue({
      status: "restored",
      database: { path: longPath, trip_count: 1 },
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
    fireEvent.click(screen.getByRole("button", { name: /^settings$/i }));
    const settings = await screen.findByRole("region", { name: "Settings" });
    const path = within(settings).getByText(longPath);
    expect(path).toHaveClass("settings-path");
    expect(path).toHaveAttribute("title", longPath);
  });
});
