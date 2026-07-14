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

  it("loads trips after successful restore with count and metadata", async () => {
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
    expect(screen.getByText("1 trip")).toBeInTheDocument();
    expect(
      within(screen.getByLabelText("Trip list")).getByText("Naha"),
    ).toBeInTheDocument();
    expect(
      within(screen.getByLabelText("Trip list")).getByText("JPY"),
    ).toBeInTheDocument();
    expect(screen.getByText(/2 days/)).toBeInTheDocument();
    expect(
      screen.getByRole("tab", { name: /Day 1 · Sun · Apr 26/i }),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/No activities planned for this day yet/),
    ).toBeInTheDocument();
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
    fireEvent.click(screen.getByRole("button", { name: /^open database$/i }));
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

  it("forgets database after confirmation without deleting wording", async () => {
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
    expect(confirmSpy.mock.calls[0]?.[0]).toMatch(/stays on disk/i);
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
    fireEvent.click(screen.getByRole("button", { name: /^open database$/i }));
    await waitFor(() =>
      expect(screen.getByText("No trips yet")).toBeInTheDocument(),
    );
    expect(screen.getByText("0 trips")).toBeInTheDocument();
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
    expect(screen.getByText("10:00")).toBeInTheDocument();
    expect(screen.getByText("Sesoko")).toBeInTheDocument();
    expect(screen.getByText(/sequence first/i)).toBeInTheDocument();
    expect(api.getDayTimeline).toHaveBeenLastCalledWith(1, 2);
  });

  it("omits empty metadata labels in trip detail", async () => {
    vi.mocked(api.restoreLastDatabase).mockResolvedValue({
      status: "restored",
      database: { path: "/tmp/sample.db", trip_count: 1 },
    });
    vi.mocked(api.listTripSummaries).mockResolvedValue([
      {
        id: 1,
        name: "Bare",
        created_at: "t",
        updated_at: "t",
      },
    ]);
    vi.mocked(api.getTripDetail).mockResolvedValue({
      id: 1,
      name: "Bare",
      created_at: "t",
      updated_at: "t",
      days: [],
    });

    render(<App />);
    await waitFor(() =>
      expect(screen.getByLabelText("Trip detail")).toHaveTextContent("Bare"),
    );
    expect(screen.queryByText("Destination")).not.toBeInTheDocument();
    expect(screen.queryByText("Currency")).not.toBeInTheDocument();
    expect(screen.queryByText("Not set")).not.toBeInTheDocument();
    expect(screen.getByText("No days yet")).toBeInTheDocument();
  });
});
