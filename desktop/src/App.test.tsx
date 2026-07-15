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
  summary: "Four days by the sea",
  main_destination: "Naha",
  main_destination_country_code: "JP",
  default_currency: "JPY",
  created_at: "t",
  updated_at: "t",
  days: [
    {
      id: 10,
      trip_id: 1,
      day_number: 1,
      date: "2026-04-26",
      title: "Arrival day",
      summary: "Settle in and explore nearby.",
    },
    {
      id: 11,
      trip_id: 1,
      day_number: 2,
      date: "2026-04-27",
      title: "Northern Okinawa",
      summary: "Aquarium and beach day.",
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
    const starting = screen.getByText("Starting…");
    expect(starting).toBeInTheDocument();
    expect(starting.closest("main")).toHaveClass("standalone-view");
    expect(
      screen
        .getByRole("heading", { level: 1, name: "Travel Ledger Desktop" })
        .closest("header"),
    ).toHaveClass("app-header");
    resolveRestore({ status: "not_found" });
    await finishBootstrap();
  });

  it("shows database-not-selected empty state with Open Database", async () => {
    render(<App />);
    await finishBootstrap();
    const emptyState = screen.getByText("Open a Travel Ledger database");
    expect(emptyState).toBeInTheDocument();
    expect(emptyState.closest("main")).toHaveClass("standalone-view");
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
    const navigator = screen.getByLabelText("Trip list sidebar");
    const settingsEntry = screen.getByRole("button", { name: /^settings$/i });
    const workspace = await screen.findByRole("region", { name: "Okinawa" });
    expect(navigator).toBeInTheDocument();
    expect(settingsEntry.closest(".sidebar-footer")).toBeInTheDocument();
    expect(workspace.closest("main")).toHaveClass("detail-pane");
    expect(
      screen
        .getByRole("heading", { level: 1, name: "Travel Ledger Desktop" })
        .closest("header"),
    ).toHaveClass("app-header");
    expect(
      screen.queryByRole("button", { name: /change database/i }),
    ).not.toBeInTheDocument();
  });

  it("renders the Trip Context Header and Plan from existing read data", async () => {
    await restoreWithSampleTrip();
    render(<App />);

    const workspace = await screen.findByRole("region", { name: "Okinawa" });
    expect(
      within(workspace).getByRole("heading", { level: 2, name: "Okinawa" }),
    ).toBeInTheDocument();
    expect(within(workspace).getByText(/2026-04-26 — 2026-04-29/)).toBeInTheDocument();
    expect(within(workspace).getByText("Naha")).toBeInTheDocument();
    expect(within(workspace).getByText("JP")).toBeInTheDocument();
    expect(within(workspace).getByText("JPY")).toBeInTheDocument();
    expect(within(workspace).getByText("Four days by the sea")).toBeInTheDocument();

    const plan = within(workspace).getByRole("region", { name: "Plan" });
    expect(within(plan).getByRole("group", { name: "Days" })).toBeInTheDocument();
    expect(
      within(plan).getByRole("heading", { level: 3, name: "Plan" }),
    ).toBeInTheDocument();
    expect(
      within(plan).getByRole("heading", {
        level: 4,
        name: "Day 1 · Sun · Apr 26",
      }),
    ).toBeInTheDocument();
    expect(within(plan).getByText("Arrival day")).toBeInTheDocument();
    expect(
      within(plan).getByText("Settle in and explore nearby."),
    ).toBeInTheDocument();
    expect(
      within(plan).getAllByRole("heading", { name: /^Day 1/ }),
    ).toHaveLength(1);
  });

  it("omits empty optional Trip and Day metadata without placeholders", async () => {
    await restoreWithSampleTrip();
    vi.mocked(api.getTripDetail).mockResolvedValue({
      ...sampleDetail,
      summary: "   ",
      main_destination: null,
      main_destination_country_code: "",
      default_currency: "  ",
      days: [
        {
          ...sampleDetail.days[0],
          title: "   ",
          summary: null,
        },
      ],
    });

    render(<App />);
    const workspace = await screen.findByRole("region", { name: "Okinawa" });
    expect(within(workspace).queryByText("Destination")).not.toBeInTheDocument();
    expect(within(workspace).queryByText("Country")).not.toBeInTheDocument();
    expect(within(workspace).queryByText("Currency")).not.toBeInTheDocument();
    expect(within(workspace).queryByLabelText("Trip summary")).not.toBeInTheDocument();
    expect(within(workspace).queryByText("Arrival day")).not.toBeInTheDocument();
    expect(
      within(workspace).queryByText("Settle in and explore nearby."),
    ).not.toBeInTheDocument();
    expect(within(workspace).queryByText("null")).not.toBeInTheDocument();
    expect(within(workspace).queryByText(/Not set/i)).not.toBeInTheDocument();
  });

  it("opens Settings with Database and About details", async () => {
    await restoreWithSampleTrip();
    render(<App />);
    await waitFor(() =>
      expect(screen.getByRole("button", { name: /^settings$/i })).toBeInTheDocument(),
    );

    fireEvent.click(screen.getByRole("button", { name: /^settings$/i }));
    const settings = await screen.findByRole("region", { name: "Settings" });
    expect(settings.closest("main")).toHaveClass("settings-view");
    expect(within(settings).getByRole("heading", { name: "Database" })).toBeInTheDocument();
    expect(within(settings).getByRole("heading", { name: "About" })).toBeInTheDocument();
    expect(
      within(settings).getByRole("button", { name: /back to trips/i }),
    ).toBeInTheDocument();
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
    expect(screen.queryByLabelText("Trip list sidebar")).not.toBeInTheDocument();
    expect(screen.queryByLabelText("Trip list")).not.toBeInTheDocument();
    expect(screen.queryByText("Okinawa")).not.toBeInTheDocument();
    expect(screen.queryByRole("region", { name: "Plan" })).not.toBeInTheDocument();
    expect(screen.queryByRole("group", { name: "Days" })).not.toBeInTheDocument();
    expect(
      screen.queryByRole("region", { name: "Itinerary timeline" }),
    ).not.toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: /^settings$/i }),
    ).not.toBeInTheDocument();
  });

  it("preserves Trip, Day, and timeline without refetching after Settings", async () => {
    await restoreWithSampleTrip();
    vi.mocked(api.getDayTimeline)
      .mockResolvedValueOnce(emptyTimeline)
      .mockResolvedValueOnce({
        ...emptyTimeline,
        day_id: 11,
        day_number: 2,
        date: "2026-04-27",
        title: "Northern Okinawa",
        summary: "Aquarium and beach day.",
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

    fireEvent.click(
      screen.getByRole("button", { name: /Day 2 · Mon · Apr 27/i }),
    );
    await screen.findByText("Beach");
    expect(
      screen.getByRole("button", { name: /Day 2 · Mon · Apr 27/i }),
    ).toHaveAttribute("aria-pressed", "true");

    fireEvent.click(screen.getByRole("button", { name: /^settings$/i }));
    expect(screen.getByRole("region", { name: "Settings" })).toBeInTheDocument();
    expect(screen.queryByLabelText("Trip list")).not.toBeInTheDocument();
    expect(screen.queryByRole("region", { name: "Plan" })).not.toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: /back to trips/i }));
    await waitFor(() =>
      expect(screen.getByRole("region", { name: "Okinawa" })).toBeInTheDocument(),
    );
    expect(screen.getByText("Beach")).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /Day 2 · Mon · Apr 27/i }),
    ).toHaveAttribute("aria-pressed", "true");
    expect(api.getTripDetail).toHaveBeenCalledTimes(1);
    expect(api.getDayTimeline).toHaveBeenCalledTimes(2);
    expect(api.listTripSummaries).toHaveBeenCalledTimes(1);
  });

  it("shows restore warning when saved DB is invalid", async () => {
    vi.mocked(api.restoreLastDatabase).mockResolvedValue({
      status: "invalid_cleared",
      code: "DATABASE_PATH_INVALID",
      message: "Database file does not exist",
    });
    render(<App />);
    await finishBootstrap();
    const alert = screen.getByRole("alert");
    const emptyState = screen.getByText("Open a Travel Ledger database");
    expect(alert).toHaveTextContent("DATABASE_PATH_INVALID");
    expect(alert.closest(".notice-area")).toBeInTheDocument();
    expect(emptyState.closest("main")).toHaveClass("standalone-view");
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

    fireEvent.click(
      screen.getByRole("button", { name: /Day 2 · Mon · Apr 27/i }),
    );
    await waitFor(() =>
      expect(
        screen.getByRole("button", { name: /Day 2 · Mon · Apr 27/i }),
      ).toHaveAttribute("aria-pressed", "true"),
    );
    fireEvent.click(screen.getByRole("button", { name: /^settings$/i }));
    fireEvent.click(screen.getByRole("button", { name: /change database/i }));
    await waitFor(() =>
      expect(screen.getByRole("alert")).toHaveTextContent("DATABASE_OPEN_FAILED"),
    );
    expect(screen.getAllByText("sample.db").length).toBeGreaterThan(0);
    expect(screen.queryByLabelText("Trip list")).not.toBeInTheDocument();
    expect(screen.getByRole("region", { name: "Settings" })).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: /back to trips/i }));
    expect(
      screen.getByRole("button", { name: /Day 2 · Mon · Apr 27/i }),
    ).toHaveAttribute("aria-pressed", "true");
  });

  it("keeps Settings and the current selection when Change is cancelled", async () => {
    await restoreWithSampleTrip();
    vi.mocked(open).mockResolvedValue(null);

    render(<App />);
    await screen.findByRole("region", { name: "Okinawa" });
    fireEvent.click(screen.getByRole("button", { name: /^settings$/i }));
    fireEvent.click(screen.getByRole("button", { name: /change database/i }));

    await waitFor(() => expect(open).toHaveBeenCalledTimes(1));
    expect(screen.getByRole("region", { name: "Settings" })).toBeInTheDocument();
    expect(screen.getAllByText("sample.db").length).toBeGreaterThan(0);
    expect(screen.queryByLabelText("Trip list")).not.toBeInTheDocument();
    expect(api.listTripSummaries).toHaveBeenCalledTimes(1);
    expect(api.getTripDetail).toHaveBeenCalledTimes(1);
    expect(api.getDayTimeline).toHaveBeenCalledTimes(1);

    fireEvent.click(screen.getByRole("button", { name: /back to trips/i }));
    expect(
      screen.getByRole("heading", { level: 2, name: "Okinawa" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /Day 1 · Sun · Apr 26/i }),
    ).toHaveAttribute("aria-pressed", "true");
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
    expect(screen.queryByLabelText("Trip list")).not.toBeInTheDocument();
    expect(screen.queryByText("Hawaii")).not.toBeInTheDocument();
    expect(screen.getByRole("region", { name: "Settings" })).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: /back to trips/i }));
    expect(
      screen.getByRole("heading", { level: 2, name: "Hawaii" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /Day 1 · Wed · Jul 1/i }),
    ).toHaveAttribute("aria-pressed", "true");
    expect(api.getDayTimeline).toHaveBeenLastCalledWith(2, 1);
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

  it("shows a Plan empty state when a Trip has no Days", async () => {
    await restoreWithSampleTrip();
    vi.mocked(api.getTripDetail).mockResolvedValue({
      ...sampleDetail,
      days: [],
    });

    render(<App />);
    const plan = await screen.findByRole("region", { name: "Plan" });
    expect(within(plan).getByText("No days yet")).toBeInTheDocument();
    expect(
      within(plan).queryByText("No activities planned for this day yet."),
    ).not.toBeInTheDocument();
    expect(api.getDayTimeline).not.toHaveBeenCalled();
  });

  it("shows the selected Day before the empty itinerary state", async () => {
    await restoreWithSampleTrip();
    render(<App />);

    const plan = await screen.findByRole("region", { name: "Plan" });
    expect(
      within(plan).getByRole("heading", { name: "Day 1 · Sun · Apr 26" }),
    ).toBeInTheDocument();
    expect(within(plan).getByText("Arrival day")).toBeInTheDocument();
    expect(
      within(plan).getByText("No activities planned for this day yet."),
    ).toBeInTheDocument();
  });

  it("renders itineraries in the order returned by the read API", async () => {
    await restoreWithSampleTrip();
    vi.mocked(api.getDayTimeline).mockResolvedValue({
      ...emptyTimeline,
      itineraries: [
        {
          id: 2,
          trip_id: 1,
          day_number: 1,
          title: "First in plan order",
          sort_order: 10,
          created_at: "t",
          updated_at: "t",
        },
        {
          id: 1,
          trip_id: 1,
          day_number: 1,
          title: "Second in plan order",
          sort_order: 20,
          created_at: "t",
          updated_at: "t",
        },
      ],
    });

    render(<App />);
    const timeline = await screen.findByRole("region", {
      name: "Itinerary timeline",
    });
    expect(
      within(timeline)
        .getAllByRole("listitem")
        .map((item) => item.textContent),
    ).toEqual(["1First in plan order", "2Second in plan order"]);
  });

  it("updates timeline when a Day button is selected", async () => {
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

    const firstDay = screen.getByRole("button", {
      name: /Day 1 · Sun · Apr 26/i,
    });
    const secondDay = screen.getByRole("button", {
      name: /Day 2 · Mon · Apr 27/i,
    });
    expect(firstDay).toHaveAttribute("aria-pressed", "true");
    expect(secondDay).toHaveAttribute("aria-pressed", "false");

    fireEvent.click(secondDay);
    await waitFor(() => expect(screen.getByText("Beach")).toBeInTheDocument());
    expect(secondDay).toHaveAttribute("aria-pressed", "true");
    expect(api.getDayTimeline).toHaveBeenLastCalledWith(1, 2);

    fireEvent.click(secondDay);
    expect(api.getDayTimeline).toHaveBeenCalledTimes(2);
  });

  it("does not show the empty timeline while activities are loading", async () => {
    await restoreWithSampleTrip();
    let resolveTimeline: (value: typeof emptyTimeline) => void = () => {};
    vi.mocked(api.getDayTimeline)
      .mockResolvedValueOnce(emptyTimeline)
      .mockReturnValueOnce(
        new Promise((resolve) => {
          resolveTimeline = resolve;
        }),
      );

    render(<App />);
    await screen.findByRole("region", { name: "Plan" });
    fireEvent.click(
      screen.getByRole("button", { name: /Day 2 · Mon · Apr 27/i }),
    );
    expect(await screen.findByRole("status")).toHaveTextContent(
      "Loading activities…",
    );
    expect(
      screen.queryByText("No activities planned for this day yet."),
    ).not.toBeInTheDocument();

    resolveTimeline({
      ...emptyTimeline,
      day_id: 11,
      day_number: 2,
      date: "2026-04-27",
    });
    expect(
      await screen.findByText("No activities planned for this day yet."),
    ).toBeInTheDocument();
  });

  it("clears the old timeline and avoids an empty-state claim on timeline error", async () => {
    await restoreWithSampleTrip();
    vi.mocked(api.getDayTimeline)
      .mockResolvedValueOnce({
        ...emptyTimeline,
        itineraries: [
          {
            id: 4,
            trip_id: 1,
            day_number: 1,
            title: "Old activity",
            sort_order: 1,
            created_at: "t",
            updated_at: "t",
          },
        ],
      })
      .mockRejectedValueOnce({
        code: "STORAGE_FAILURE",
        message: "timeline failed",
      });

    render(<App />);
    await screen.findByText("Old activity");
    fireEvent.click(
      screen.getByRole("button", { name: /Day 2 · Mon · Apr 27/i }),
    );

    await waitFor(() =>
      expect(screen.getByRole("alert")).toHaveTextContent("timeline failed"),
    );
    expect(screen.getByRole("region", { name: "Okinawa" })).toBeInTheDocument();
    expect(screen.queryByText("Old activity")).not.toBeInTheDocument();
    expect(
      screen.queryByText("No activities planned for this day yet."),
    ).not.toBeInTheDocument();
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
