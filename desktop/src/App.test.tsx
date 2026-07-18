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
  createItinerary: vi.fn(),
  reorderItinerary: vi.fn(),
  updateItinerary: vi.fn(),
  createTrip: vi.fn(),
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

const sampleActivity = {
  id: 20,
  trip_id: 1,
  day_number: 1,
  title: "Museum visit",
  note: "Buy tickets",
  start_time: "09:30",
  sort_order: 1000,
  duration_minutes: 90,
  travel_minutes: 15,
  location: "Naha",
  category: "museum",
  created_at: "created",
  updated_at: "updated-1",
};

const activityTimeline = {
  ...emptyTimeline,
  itineraries: [sampleActivity],
};

const reorderTimeline = {
  ...emptyTimeline,
  itineraries: [
    sampleActivity,
    {
      ...sampleActivity,
      id: 21,
      title: "Lunch",
      note: null,
      start_time: "12:00",
      sort_order: 2000,
      location: "Market",
      category: "meal",
    },
    {
      ...sampleActivity,
      id: 22,
      title: "Beach",
      note: null,
      start_time: "15:00",
      sort_order: 3000,
      location: "Naminoue",
      category: "sightseeing",
    },
  ],
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
  vi.mocked(api.createItinerary).mockResolvedValue({ itinerary_id: 20 });
  vi.mocked(api.reorderItinerary).mockResolvedValue({
    itinerary_id: 20,
    day_number: 1,
    moved: true,
  });
  vi.mocked(api.updateItinerary).mockResolvedValue({ itinerary_id: 20 });
  vi.mocked(api.createTrip).mockResolvedValue({ trip_id: 2 });
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
    expect(
      within(settings).getByText("Access: local Trip creation"),
    ).toBeInTheDocument();
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
    expect(screen.getByRole("button", { name: /new trip/i })).toBeInTheDocument();
  });

  it("shows New Trip only when a database is selected", async () => {
    render(<App />);
    await finishBootstrap();
    expect(
      screen.queryByRole("button", { name: /new trip/i }),
    ).not.toBeInTheDocument();

    await restoreWithSampleTrip();
  });

  it("opens the Trip creation form with all fields", async () => {
    await restoreWithSampleTrip();
    render(<App />);
    await screen.findByRole("region", { name: "Okinawa" });

    const newTrip = screen.getByRole("button", { name: /new trip/i });
    fireEvent.click(newTrip);
    expect(newTrip).toHaveAttribute("aria-pressed", "true");
    expect(
      screen.getByRole("region", { name: "Create a new trip" }),
    ).toBeInTheDocument();
    expect(screen.getByLabelText("Trip name")).toBeRequired();
    expect(screen.getByLabelText("Start date")).toHaveAttribute("type", "date");
    expect(screen.getByLabelText("End date")).toHaveAttribute("type", "date");
    expect(screen.getByLabelText("Summary")).toBeInTheDocument();
    expect(screen.getByLabelText("Main destination")).toBeInTheDocument();
    expect(
      screen.getByLabelText("Main destination country code"),
    ).toBeInTheDocument();
    expect(screen.getByLabelText("Default currency")).toBeInTheDocument();
    expect(
      within(screen.getByLabelText("Trip list")).getByRole("button", {
        name: /Okinawa/i,
      }),
    ).toHaveAttribute("aria-pressed", "false");
  });

  it("cancels creation without refetching the preserved Trip and Day", async () => {
    await restoreWithSampleTrip();
    vi.mocked(api.getDayTimeline)
      .mockResolvedValueOnce(emptyTimeline)
      .mockResolvedValueOnce({
        ...emptyTimeline,
        day_id: 11,
        day_number: 2,
        date: "2026-04-27",
      });
    render(<App />);
    await screen.findByRole("region", { name: "Okinawa" });
    fireEvent.click(
      screen.getByRole("button", { name: /Day 2 · Mon · Apr 27/i }),
    );
    await waitFor(() =>
      expect(
        screen.getByRole("button", { name: /Day 2 · Mon · Apr 27/i }),
      ).toHaveAttribute("aria-pressed", "true"),
    );

    fireEvent.click(screen.getByRole("button", { name: /new trip/i }));
    fireEvent.change(screen.getByLabelText("Trip name"), {
      target: { value: "Draft" },
    });
    fireEvent.click(screen.getByRole("button", { name: /cancel/i }));

    expect(screen.getByRole("region", { name: "Okinawa" })).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /Day 2 · Mon · Apr 27/i }),
    ).toHaveAttribute("aria-pressed", "true");
    expect(api.listTripSummaries).toHaveBeenCalledTimes(1);
    expect(api.getTripDetail).toHaveBeenCalledTimes(1);
    expect(api.getDayTimeline).toHaveBeenCalledTimes(2);
  });

  it("leaves creation for an existing Trip or Settings without extra reads", async () => {
    await restoreWithSampleTrip();
    render(<App />);
    await screen.findByRole("region", { name: "Okinawa" });

    fireEvent.click(screen.getByRole("button", { name: /new trip/i }));
    fireEvent.click(
      within(screen.getByLabelText("Trip list")).getByRole("button", {
        name: /Okinawa/i,
      }),
    );
    expect(screen.getByRole("region", { name: "Okinawa" })).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: /new trip/i }));
    fireEvent.click(screen.getByRole("button", { name: /^settings$/i }));
    expect(screen.getByRole("region", { name: "Settings" })).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: /back to trips/i }));
    expect(screen.getByRole("region", { name: "Okinawa" })).toBeInTheDocument();
    expect(api.listTripSummaries).toHaveBeenCalledTimes(1);
    expect(api.getTripDetail).toHaveBeenCalledTimes(1);
    expect(api.getDayTimeline).toHaveBeenCalledTimes(1);
  });

  it("creates a Trip, refreshes from the database, and selects Day 1", async () => {
    await restoreWithSampleTrip();
    const createdSummary = {
      id: 2,
      name: "Kyoto",
      start_date: "2026-08-01",
      end_date: "2026-08-03",
      main_destination: "Kyoto",
      main_destination_country_code: "JP",
      default_currency: "JPY",
      created_at: "t2",
      updated_at: "t2",
    };
    const createdDetail = {
      ...createdSummary,
      summary: "Temples",
      days: [
        {
          id: 20,
          trip_id: 2,
          day_number: 1,
          date: "2026-08-01",
          title: "",
          summary: null,
        },
      ],
    };
    vi.mocked(api.listTripSummaries)
      .mockResolvedValueOnce(sampleTrips)
      .mockResolvedValueOnce([sampleTrips[0], createdSummary]);
    vi.mocked(api.getTripDetail)
      .mockResolvedValueOnce(sampleDetail)
      .mockResolvedValueOnce(createdDetail);
    vi.mocked(api.getDayTimeline)
      .mockResolvedValueOnce(emptyTimeline)
      .mockResolvedValueOnce({
        ...emptyTimeline,
        trip_id: 2,
        trip_name: "Kyoto",
        day_id: 20,
        date: "2026-08-01",
      });

    render(<App />);
    await screen.findByRole("region", { name: "Okinawa" });
    fireEvent.click(screen.getByRole("button", { name: /new trip/i }));
    fireEvent.change(screen.getByLabelText("Trip name"), {
      target: { value: "Kyoto" },
    });
    fireEvent.change(screen.getByLabelText("Start date"), {
      target: { value: "2026-08-01" },
    });
    fireEvent.change(screen.getByLabelText("End date"), {
      target: { value: "2026-08-03" },
    });
    fireEvent.change(screen.getByLabelText("Summary"), {
      target: { value: "Temples" },
    });
    fireEvent.change(screen.getByLabelText("Main destination"), {
      target: { value: "Kyoto" },
    });
    fireEvent.change(screen.getByLabelText("Main destination country code"), {
      target: { value: "jp" },
    });
    fireEvent.change(screen.getByLabelText("Default currency"), {
      target: { value: "jpy" },
    });
    fireEvent.click(screen.getByRole("button", { name: /create trip/i }));

    expect(await screen.findByRole("region", { name: "Kyoto" })).toBeInTheDocument();
    expect(api.createTrip).toHaveBeenCalledWith({
      name: "Kyoto",
      start_date: "2026-08-01",
      end_date: "2026-08-03",
      summary: "Temples",
      main_destination: "Kyoto",
      main_destination_country_code: "jp",
      default_currency: "jpy",
    });
    expect(api.listTripSummaries).toHaveBeenCalledTimes(2);
    expect(api.getTripDetail).toHaveBeenLastCalledWith(2);
    expect(api.getDayTimeline).toHaveBeenLastCalledWith(2, 1);
    expect(
      screen.getByRole("button", { name: /Day 1 · Sat · Aug 1/i }),
    ).toHaveAttribute("aria-pressed", "true");
    expect(
      within(screen.getByLabelText("Trip list")).getByRole("button", {
        name: /Kyoto/i,
      }),
    ).toHaveAttribute("aria-pressed", "true");
  });

  it("prevents duplicate submit while creation is pending", async () => {
    await restoreWithSampleTrip();
    vi.mocked(api.createTrip).mockReturnValue(new Promise(() => {}));
    render(<App />);
    await screen.findByRole("region", { name: "Okinawa" });
    fireEvent.click(screen.getByRole("button", { name: /new trip/i }));
    fireEvent.change(screen.getByLabelText("Trip name"), {
      target: { value: "Pending" },
    });
    fireEvent.change(screen.getByLabelText("Start date"), {
      target: { value: "2026-08-01" },
    });
    fireEvent.change(screen.getByLabelText("End date"), {
      target: { value: "2026-08-01" },
    });
    const form = screen.getByLabelText("Trip name").closest("form");
    expect(form).not.toBeNull();
    fireEvent.submit(form!);
    fireEvent.submit(form!);
    expect(api.createTrip).toHaveBeenCalledTimes(1);
    expect(screen.getByRole("button", { name: /creating/i })).toBeDisabled();
    expect(screen.getByRole("button", { name: /cancel/i })).toBeDisabled();
  });

  it("blocks a reversed date range before calling the write API", async () => {
    await restoreWithSampleTrip();
    render(<App />);
    await screen.findByRole("region", { name: "Okinawa" });
    fireEvent.click(screen.getByRole("button", { name: /new trip/i }));
    fireEvent.change(screen.getByLabelText("Trip name"), {
      target: { value: "Bad Range" },
    });
    fireEvent.change(screen.getByLabelText("Start date"), {
      target: { value: "2026-08-03" },
    });
    fireEvent.change(screen.getByLabelText("End date"), {
      target: { value: "2026-08-01" },
    });
    fireEvent.click(screen.getByRole("button", { name: /create trip/i }));

    expect(screen.getByRole("alert")).toHaveTextContent(
      "End date must be on or after start date.",
    );
    expect(api.createTrip).not.toHaveBeenCalled();
  });

  it("creates the first Trip in an empty database", async () => {
    vi.mocked(api.restoreLastDatabase).mockResolvedValue({
      status: "restored",
      database: { path: "/tmp/empty.db", trip_count: 0 },
    });
    const createdSummary = {
      id: 1,
      name: "First Trip",
      start_date: "2026-09-01",
      end_date: "2026-09-01",
      created_at: "t",
      updated_at: "t",
    };
    vi.mocked(api.listTripSummaries)
      .mockResolvedValueOnce([])
      .mockResolvedValueOnce([createdSummary]);
    vi.mocked(api.createTrip).mockResolvedValue({ trip_id: 1 });
    vi.mocked(api.getTripDetail).mockResolvedValue({
      ...createdSummary,
      days: [
        {
          id: 10,
          trip_id: 1,
          day_number: 1,
          date: "2026-09-01",
          title: "",
          summary: null,
        },
      ],
    });
    vi.mocked(api.getDayTimeline).mockResolvedValue({
      ...emptyTimeline,
      trip_name: "First Trip",
      date: "2026-09-01",
    });

    render(<App />);
    await screen.findByText("No trips yet");
    fireEvent.click(screen.getByRole("button", { name: /new trip/i }));
    fireEvent.change(screen.getByLabelText("Trip name"), {
      target: { value: "First Trip" },
    });
    fireEvent.change(screen.getByLabelText("Start date"), {
      target: { value: "2026-09-01" },
    });
    fireEvent.change(screen.getByLabelText("End date"), {
      target: { value: "2026-09-01" },
    });
    fireEvent.click(screen.getByRole("button", { name: /create trip/i }));

    expect(
      await screen.findByRole("region", { name: "First Trip" }),
    ).toBeInTheDocument();
    expect(api.createTrip).toHaveBeenCalledWith({
      name: "First Trip",
      start_date: "2026-09-01",
      end_date: "2026-09-01",
      summary: null,
      main_destination: null,
      main_destination_country_code: null,
      default_currency: null,
    });
    expect(api.getTripDetail).toHaveBeenCalledWith(1);
    expect(api.getDayTimeline).toHaveBeenCalledWith(1, 1);
  });

  it("keeps form input after write failure", async () => {
    await restoreWithSampleTrip();
    vi.mocked(api.createTrip).mockRejectedValue({
      code: "TRIP_VALIDATION_FAILED",
      message: "invalid country code",
    });
    render(<App />);
    await screen.findByRole("region", { name: "Okinawa" });
    fireEvent.click(screen.getByRole("button", { name: /new trip/i }));
    fireEvent.change(screen.getByLabelText("Trip name"), {
      target: { value: "Keep this input" },
    });
    fireEvent.change(screen.getByLabelText("Start date"), {
      target: { value: "2026-08-01" },
    });
    fireEvent.change(screen.getByLabelText("End date"), {
      target: { value: "2026-08-02" },
    });
    fireEvent.change(screen.getByLabelText("Main destination country code"), {
      target: { value: "XX" },
    });
    fireEvent.click(screen.getByRole("button", { name: /create trip/i }));

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "invalid country code",
    );
    expect(screen.getByLabelText("Trip name")).toHaveValue("Keep this input");
    expect(screen.getByLabelText("Main destination country code")).toHaveValue(
      "XX",
    );
    expect(api.listTripSummaries).toHaveBeenCalledTimes(1);
    expect(api.getTripDetail).toHaveBeenCalledTimes(1);
  });

  it("closes the form after committed write even if refresh fails", async () => {
    await restoreWithSampleTrip();
    vi.mocked(api.listTripSummaries)
      .mockResolvedValueOnce(sampleTrips)
      .mockRejectedValueOnce({
        code: "STORAGE_FAILURE",
        message: "refresh failed after commit",
      });
    render(<App />);
    await screen.findByRole("region", { name: "Okinawa" });
    fireEvent.click(screen.getByRole("button", { name: /new trip/i }));
    fireEvent.change(screen.getByLabelText("Trip name"), {
      target: { value: "Committed" },
    });
    fireEvent.change(screen.getByLabelText("Start date"), {
      target: { value: "2026-08-01" },
    });
    fireEvent.change(screen.getByLabelText("End date"), {
      target: { value: "2026-08-01" },
    });
    fireEvent.click(screen.getByRole("button", { name: /create trip/i }));

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "refresh failed after commit",
    );
    expect(
      screen.queryByRole("region", { name: "Create a new trip" }),
    ).not.toBeInTheDocument();
    expect(api.createTrip).toHaveBeenCalledTimes(1);
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
        .getAllByText(/in plan order$/)
        .map((item) => item.textContent),
    ).toEqual(["First in plan order", "Second in plan order"]);
  });

  it("exposes one-step reorder controls with boundary buttons disabled", async () => {
    await restoreWithSampleTrip();
    vi.mocked(api.getDayTimeline).mockResolvedValue(reorderTimeline);

    render(<App />);

    expect(
      await screen.findByRole("button", {
        name: "Move activity up: Museum visit",
      }),
    ).toBeDisabled();
    expect(
      screen.getByRole("button", {
        name: "Move activity down: Museum visit",
      }),
    ).toBeEnabled();
    expect(
      screen.getByRole("button", { name: "Move activity up: Lunch" }),
    ).toBeEnabled();
    expect(
      screen.getByRole("button", { name: "Move activity down: Lunch" }),
    ).toBeEnabled();
    expect(
      screen.getByRole("button", { name: "Move activity up: Beach" }),
    ).toBeEnabled();
    expect(
      screen.getByRole("button", { name: "Move activity down: Beach" }),
    ).toBeDisabled();
  });

  it("disables both reorder directions for a one-activity Day", async () => {
    await restoreWithSampleTrip();
    vi.mocked(api.getDayTimeline).mockResolvedValue(activityTimeline);

    render(<App />);

    expect(
      await screen.findByRole("button", {
        name: "Move activity up: Museum visit",
      }),
    ).toBeDisabled();
    expect(
      screen.getByRole("button", {
        name: "Move activity down: Museum visit",
      }),
    ).toBeDisabled();
  });

  it("moves one activity, refreshes only the selected Day, and restores focus", async () => {
    await restoreWithSampleTrip();
    vi.mocked(api.reorderItinerary).mockResolvedValue({
      itinerary_id: 21,
      day_number: 1,
      moved: true,
    });
    vi.mocked(api.getDayTimeline)
      .mockResolvedValueOnce(reorderTimeline)
      .mockResolvedValueOnce({
        ...reorderTimeline,
        itineraries: [
          { ...reorderTimeline.itineraries[1], sort_order: 1000 },
          { ...reorderTimeline.itineraries[0], sort_order: 2000 },
          reorderTimeline.itineraries[2],
        ],
      });

    render(<App />);
    const moveUp = await screen.findByRole("button", {
      name: "Move activity up: Lunch",
    });
    fireEvent.click(moveUp);

    await waitFor(() =>
      expect(api.reorderItinerary).toHaveBeenCalledWith({
        trip_id: 1,
        day_number: 1,
        itinerary_id: 21,
        direction: "up",
        expected_order: [20, 21, 22],
      }),
    );
    const timeline = await screen.findByRole("region", {
      name: "Itinerary timeline",
    });
    await waitFor(() =>
      expect(
        within(timeline)
          .getAllByText(/^(Lunch|Museum visit|Beach)$/)
          .map((item) => item.textContent),
      ).toEqual(["Lunch", "Museum visit", "Beach"]),
    );
    expect(api.getDayTimeline).toHaveBeenCalledTimes(2);
    expect(api.getTripDetail).toHaveBeenCalledTimes(1);
    expect(api.listTripSummaries).toHaveBeenCalledTimes(1);
    expect(screen.getByRole("status")).toHaveTextContent("Moved Lunch up.");
    await waitFor(() =>
      expect(
        screen.getByRole("button", { name: "Move activity down: Lunch" }),
      ).toHaveFocus(),
    );
  });

  it("locks all reorder controls immediately during a pending write", async () => {
    await restoreWithSampleTrip();
    vi.mocked(api.getDayTimeline).mockResolvedValue(reorderTimeline);
    let resolveReorder: (
      value: { itinerary_id: number; day_number: number; moved: boolean },
    ) => void = () => {};
    vi.mocked(api.reorderItinerary).mockReturnValue(
      new Promise((resolve) => {
        resolveReorder = resolve;
      }),
    );

    render(<App />);
    const moveDown = await screen.findByRole("button", {
      name: "Move activity down: Museum visit",
    });
    fireEvent.click(moveDown);
    fireEvent.click(moveDown);

    expect(api.reorderItinerary).toHaveBeenCalledTimes(1);
    expect(
      screen.getByRole("region", { name: "Itinerary timeline" }),
    ).toHaveAttribute("aria-busy", "true");
    expect(
      screen
        .getAllByRole("button", { name: /Move activity (up|down):/ })
        .every((button) => (button as HTMLButtonElement).disabled),
    ).toBe(true);

    resolveReorder({ itinerary_id: 20, day_number: 1, moved: true });
    await waitFor(() => expect(api.getDayTimeline).toHaveBeenCalledTimes(2));
  });

  it("preserves an open Inspector draft while the selected activity moves", async () => {
    await restoreWithSampleTrip();
    vi.mocked(api.getDayTimeline)
      .mockResolvedValueOnce(reorderTimeline)
      .mockResolvedValueOnce({
        ...reorderTimeline,
        itineraries: [
          { ...reorderTimeline.itineraries[1], sort_order: 1000 },
          {
            ...reorderTimeline.itineraries[0],
            sort_order: 2000,
            updated_at: "updated-by-reorder",
          },
          reorderTimeline.itineraries[2],
        ],
      });

    render(<App />);
    fireEvent.click(
      await screen.findByRole("button", { name: "Edit activity: Museum visit" }),
    );
    const inspector = screen.getByRole("region", { name: "Edit activity" });
    fireEvent.change(within(inspector).getByLabelText("Title"), {
      target: { value: "Unsaved museum draft" },
    });
    fireEvent.click(
      screen.getByRole("button", {
        name: "Move activity down: Museum visit",
      }),
    );

    await waitFor(() => expect(api.getDayTimeline).toHaveBeenCalledTimes(2));
    expect(screen.getByRole("region", { name: "Edit activity" })).toBe(inspector);
    expect(within(inspector).getByLabelText("Title")).toHaveValue(
      "Unsaved museum draft",
    );
  });

  it("keeps the current timeline on reorder write failure", async () => {
    await restoreWithSampleTrip();
    vi.mocked(api.getDayTimeline).mockResolvedValue(reorderTimeline);
    vi.mocked(api.reorderItinerary).mockRejectedValue({
      code: "ITINERARY_PLACEMENT_CONFLICT",
      message: "Activity order changed. Try again.",
    });

    render(<App />);
    fireEvent.click(
      await screen.findByRole("button", { name: "Move activity up: Lunch" }),
    );

    await waitFor(() =>
      expect(screen.getByRole("alert")).toHaveTextContent(
        "Activity order changed. Try again.",
      ),
    );
    expect(api.getDayTimeline).toHaveBeenCalledTimes(1);
    expect(screen.getByText("Museum visit")).toBeInTheDocument();
    expect(screen.getByText("Lunch")).toBeInTheDocument();
  });

  it("separates a committed reorder from timeline refresh failure", async () => {
    await restoreWithSampleTrip();
    vi.mocked(api.getDayTimeline)
      .mockResolvedValueOnce(reorderTimeline)
      .mockRejectedValueOnce({
        code: "STORAGE_FAILURE",
        message: "reorder refresh failed",
      });

    render(<App />);
    fireEvent.click(
      await screen.findByRole("button", { name: "Edit activity: Museum visit" }),
    );
    fireEvent.click(
      screen.getByRole("button", { name: "Move activity down: Museum visit" }),
    );

    await waitFor(() =>
      expect(screen.getByRole("alert")).toHaveTextContent(
        "reorder refresh failed",
      ),
    );
    expect(api.reorderItinerary).toHaveBeenCalledTimes(1);
    expect(screen.queryByText("Museum visit")).not.toBeInTheDocument();
    expect(screen.queryByRole("region", { name: "Edit activity" })).not.toBeInTheDocument();
  });

  it("ignores a stale reorder completion after Day navigation", async () => {
    await restoreWithSampleTrip();
    let resolveReorder: (
      value: { itinerary_id: number; day_number: number; moved: boolean },
    ) => void = () => {};
    vi.mocked(api.reorderItinerary).mockReturnValue(
      new Promise((resolve) => {
        resolveReorder = resolve;
      }),
    );
    vi.mocked(api.getDayTimeline)
      .mockResolvedValueOnce(reorderTimeline)
      .mockResolvedValueOnce({
        ...emptyTimeline,
        day_id: 11,
        day_number: 2,
        date: "2026-04-27",
        itineraries: [
          {
            ...sampleActivity,
            id: 30,
            day_number: 2,
            title: "Day 2 activity",
          },
        ],
      });

    render(<App />);
    fireEvent.click(
      await screen.findByRole("button", { name: "Move activity up: Lunch" }),
    );
    fireEvent.click(
      screen.getByRole("button", { name: /Day 2 · Mon · Apr 27/i }),
    );
    expect(await screen.findByText("Day 2 activity")).toBeInTheDocument();

    resolveReorder({ itinerary_id: 21, day_number: 1, moved: true });
    await waitFor(() => expect(api.reorderItinerary).toHaveBeenCalledTimes(1));
    expect(api.getDayTimeline).toHaveBeenCalledTimes(2);
    expect(screen.getByText("Day 2 activity")).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /Day 2 · Mon · Apr 27/i }),
    ).toHaveAttribute("aria-pressed", "true");
  });

  it("disables reorder while quick add is open or an Inspector save is pending", async () => {
    await restoreWithSampleTrip();
    vi.mocked(api.getDayTimeline).mockResolvedValue(reorderTimeline);
    let resolveUpdate: (value: { itinerary_id: number }) => void = () => {};
    vi.mocked(api.updateItinerary).mockReturnValue(
      new Promise((resolve) => {
        resolveUpdate = resolve;
      }),
    );

    render(<App />);
    await screen.findByRole("button", { name: "Move activity up: Lunch" });
    fireEvent.click(screen.getByRole("button", { name: "Add activity" }));
    expect(
      screen
        .getAllByRole("button", { name: /Move activity (up|down):/ })
        .every((button) => (button as HTMLButtonElement).disabled),
    ).toBe(true);

    fireEvent.click(
      within(screen.getByRole("region", { name: "Add activity" })).getByRole(
        "button",
        { name: "Cancel" },
      ),
    );
    fireEvent.click(
      screen.getByRole("button", { name: "Edit activity: Museum visit" }),
    );
    const inspector = screen.getByRole("region", { name: "Edit activity" });
    fireEvent.change(within(inspector).getByLabelText("Title"), {
      target: { value: "Pending edit" },
    });
    fireEvent.click(within(inspector).getByRole("button", { name: "Save" }));
    expect(
      screen
        .getAllByRole("button", { name: /Move activity (up|down):/ })
        .every((button) => (button as HTMLButtonElement).disabled),
    ).toBe(true);
    expect(api.reorderItinerary).not.toHaveBeenCalled();

    resolveUpdate({ itinerary_id: 20 });
    await waitFor(() => expect(api.getDayTimeline).toHaveBeenCalledTimes(2));
  });

  it("opens a contextual activity Inspector with a prefilled editable draft", async () => {
    await restoreWithSampleTrip();
    vi.mocked(api.getDayTimeline).mockResolvedValue(activityTimeline);

    render(<App />);
    const editButton = await screen.findByRole("button", {
      name: "Edit activity: Museum visit",
    });
    expect(editButton).toHaveAttribute("aria-expanded", "false");
    fireEvent.click(editButton);

    const inspector = screen.getByRole("region", { name: "Edit activity" });
    expect(editButton).toHaveAttribute("aria-expanded", "true");
    expect(editButton).toHaveAttribute("aria-controls", "activity-inspector");
    expect(editButton.closest("li")).toHaveClass("selected");
    expect(within(inspector).getByLabelText("Title")).toHaveValue("Museum visit");
    expect(within(inspector).getByLabelText("Start time")).toHaveValue("09:30");
    expect(within(inspector).getByLabelText("Location")).toHaveValue("Naha");
    expect(within(inspector).getByLabelText("Note")).toHaveValue("Buy tickets");
    expect(within(inspector).getByRole("button", { name: "Save" })).toBeDisabled();
    expect(within(inspector).getByLabelText("Title")).toHaveFocus();

    fireEvent.click(within(inspector).getByRole("button", { name: "Cancel" }));
    await waitFor(() => expect(editButton).toHaveFocus());
    expect(screen.queryByRole("region", { name: "Edit activity" })).not.toBeInTheDocument();
  });

  it("discards the old draft when another activity is selected", async () => {
    await restoreWithSampleTrip();
    vi.mocked(api.getDayTimeline).mockResolvedValue({
      ...activityTimeline,
      itineraries: [
        sampleActivity,
        {
          ...sampleActivity,
          id: 21,
          title: "Lunch",
          start_time: "12:00",
          location: "Market",
          note: null,
          sort_order: 2000,
        },
      ],
    });

    render(<App />);
    fireEvent.click(
      await screen.findByRole("button", { name: "Edit activity: Museum visit" }),
    );
    fireEvent.change(
      within(screen.getByRole("region", { name: "Edit activity" })).getByLabelText(
        "Title",
      ),
      { target: { value: "Discard this draft" } },
    );
    fireEvent.click(
      screen.getByRole("button", { name: "Edit activity: Lunch" }),
    );

    const inspector = screen.getByRole("region", { name: "Edit activity" });
    expect(within(inspector).getByLabelText("Title")).toHaveValue("Lunch");
    expect(within(inspector).getByLabelText("Location")).toHaveValue("Market");
    expect(
      screen.getByRole("button", { name: "Edit activity: Lunch" }),
    ).toHaveAttribute("aria-expanded", "true");
  });

  it("updates four activity fields and refreshes only the selected Day", async () => {
    await restoreWithSampleTrip();
    vi.mocked(api.getDayTimeline)
      .mockResolvedValueOnce(activityTimeline)
      .mockResolvedValueOnce({
        ...activityTimeline,
        itineraries: [
          {
            ...sampleActivity,
            title: "Morning museum",
            start_time: null,
            location: "Shuri",
            note: null,
            updated_at: "updated-2",
          },
        ],
      });

    render(<App />);
    fireEvent.click(
      await screen.findByRole("button", { name: "Edit activity: Museum visit" }),
    );
    const inspector = screen.getByRole("region", { name: "Edit activity" });
    fireEvent.change(within(inspector).getByLabelText("Title"), {
      target: { value: "  Morning museum  " },
    });
    fireEvent.change(within(inspector).getByLabelText("Start time"), {
      target: { value: "" },
    });
    fireEvent.change(within(inspector).getByLabelText("Location"), {
      target: { value: "  Shuri  " },
    });
    fireEvent.change(within(inspector).getByLabelText("Note"), {
      target: { value: "   " },
    });
    fireEvent.click(within(inspector).getByRole("button", { name: "Save" }));

    await waitFor(() =>
      expect(api.updateItinerary).toHaveBeenCalledWith({
        trip_id: 1,
        day_number: 1,
        itinerary_id: 20,
        title: "  Morning museum  ",
        start_time: null,
        location: "  Shuri  ",
        note: null,
      }),
    );
    const refreshedInspector = await screen.findByRole("region", {
      name: "Edit activity",
    });
    expect(within(refreshedInspector).getByLabelText("Title")).toHaveValue(
      "Morning museum",
    );
    expect(within(refreshedInspector).getByLabelText("Location")).toHaveValue(
      "Shuri",
    );
    expect(api.getDayTimeline).toHaveBeenCalledTimes(2);
    expect(api.getTripDetail).toHaveBeenCalledTimes(1);
    expect(api.listTripSummaries).toHaveBeenCalledTimes(1);
    expect(
      screen.getByRole("button", { name: /Day 1 · Sun · Apr 26/i }),
    ).toHaveAttribute("aria-pressed", "true");
  });

  it("closes the Inspector when refreshed data no longer contains the activity", async () => {
    await restoreWithSampleTrip();
    vi.mocked(api.getDayTimeline)
      .mockResolvedValueOnce(activityTimeline)
      .mockResolvedValueOnce(emptyTimeline);

    render(<App />);
    fireEvent.click(
      await screen.findByRole("button", { name: "Edit activity: Museum visit" }),
    );
    const inspector = screen.getByRole("region", { name: "Edit activity" });
    fireEvent.change(within(inspector).getByLabelText("Title"), {
      target: { value: "Missing after refresh" },
    });
    fireEvent.click(within(inspector).getByRole("button", { name: "Save" }));

    expect(
      await screen.findByText("No activities planned for this day yet."),
    ).toBeInTheDocument();
    expect(screen.queryByRole("region", { name: "Edit activity" })).not.toBeInTheDocument();
    expect(api.updateItinerary).toHaveBeenCalledTimes(1);
    expect(api.getDayTimeline).toHaveBeenCalledTimes(2);
  });

  it("keeps the Inspector draft on write failure and permits a retry", async () => {
    await restoreWithSampleTrip();
    vi.mocked(api.getDayTimeline).mockResolvedValue(activityTimeline);
    vi.mocked(api.updateItinerary).mockRejectedValueOnce({
      code: "ITINERARY_VALIDATION_FAILED",
      message: "Itinerary title must not be empty",
    });

    render(<App />);
    fireEvent.click(
      await screen.findByRole("button", { name: "Edit activity: Museum visit" }),
    );
    const inspector = screen.getByRole("region", { name: "Edit activity" });
    fireEvent.change(within(inspector).getByLabelText("Title"), {
      target: { value: "   " },
    });
    fireEvent.change(within(inspector).getByLabelText("Note"), {
      target: { value: "Keep this edit" },
    });
    fireEvent.click(within(inspector).getByRole("button", { name: "Save" }));

    await waitFor(() =>
      expect(screen.getByRole("alert")).toHaveTextContent(
        "Itinerary title must not be empty",
      ),
    );
    expect(within(inspector).getByLabelText("Title")).toHaveValue("   ");
    expect(within(inspector).getByLabelText("Note")).toHaveValue("Keep this edit");
    expect(api.getDayTimeline).toHaveBeenCalledTimes(1);
    await waitFor(() =>
      expect(within(inspector).getByRole("button", { name: "Save" })).toBeEnabled(),
    );
    fireEvent.click(within(inspector).getByRole("button", { name: "Save" }));
    await waitFor(() => expect(api.updateItinerary).toHaveBeenCalledTimes(2));
  });

  it("locks Inspector submission immediately", async () => {
    await restoreWithSampleTrip();
    vi.mocked(api.getDayTimeline).mockResolvedValue(activityTimeline);
    let resolveUpdate: (value: { itinerary_id: number }) => void = () => {};
    vi.mocked(api.updateItinerary).mockReturnValue(
      new Promise((resolve) => {
        resolveUpdate = resolve;
      }),
    );

    render(<App />);
    fireEvent.click(
      await screen.findByRole("button", { name: "Edit activity: Museum visit" }),
    );
    const inspector = screen.getByRole("region", { name: "Edit activity" });
    fireEvent.change(within(inspector).getByLabelText("Title"), {
      target: { value: "Changed" },
    });
    const form = within(inspector).getByRole("button", { name: "Save" }).closest("form");
    expect(form).not.toBeNull();
    fireEvent.submit(form!);
    fireEvent.submit(form!);

    expect(api.updateItinerary).toHaveBeenCalledTimes(1);
    expect(within(inspector).getByRole("button", { name: "Saving…" })).toBeDisabled();
    expect(within(inspector).getByRole("button", { name: "Cancel" })).toBeDisabled();
    resolveUpdate({ itinerary_id: 20 });
    await waitFor(() => expect(api.getDayTimeline).toHaveBeenCalledTimes(2));
  });

  it("keeps quick add and the Inspector mutually exclusive", async () => {
    await restoreWithSampleTrip();
    vi.mocked(api.getDayTimeline).mockResolvedValue(activityTimeline);

    render(<App />);
    fireEvent.click(
      await screen.findByRole("button", { name: "Edit activity: Museum visit" }),
    );
    expect(screen.getByRole("region", { name: "Edit activity" })).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Add activity" }));
    expect(screen.queryByRole("region", { name: "Edit activity" })).not.toBeInTheDocument();
    expect(screen.getByRole("region", { name: "Add activity" })).toBeInTheDocument();

    fireEvent.click(
      screen.getByRole("button", { name: "Edit activity: Museum visit" }),
    );
    expect(screen.getByRole("region", { name: "Edit activity" })).toBeInTheDocument();
    expect(screen.queryByRole("region", { name: "Add activity" })).not.toBeInTheDocument();
  });

  it("discards the Inspector draft on Day and Settings navigation", async () => {
    await restoreWithSampleTrip();
    vi.mocked(api.getDayTimeline)
      .mockResolvedValueOnce(activityTimeline)
      .mockResolvedValueOnce({
        ...emptyTimeline,
        day_id: 11,
        day_number: 2,
        date: "2026-04-27",
      })
      .mockResolvedValueOnce(activityTimeline);

    render(<App />);
    fireEvent.click(
      await screen.findByRole("button", { name: "Edit activity: Museum visit" }),
    );
    fireEvent.change(
      within(screen.getByRole("region", { name: "Edit activity" })).getByLabelText(
        "Title",
      ),
      { target: { value: "Discard me" } },
    );
    fireEvent.click(
      screen.getByRole("button", { name: /Day 2 · Mon · Apr 27/i }),
    );
    await waitFor(() =>
      expect(screen.queryByRole("region", { name: "Edit activity" })).not.toBeInTheDocument(),
    );

    fireEvent.click(
      screen.getByRole("button", { name: /Day 1 · Sun · Apr 26/i }),
    );
    fireEvent.click(
      await screen.findByRole("button", { name: "Edit activity: Museum visit" }),
    );
    fireEvent.click(screen.getByRole("button", { name: /^settings$/i }));
    expect(screen.getByRole("region", { name: "Settings" })).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: /back to trips/i }));
    expect(screen.queryByRole("region", { name: "Edit activity" })).not.toBeInTheDocument();
  });

  it("discards the Inspector draft when New Trip replaces the Workspace", async () => {
    await restoreWithSampleTrip();
    vi.mocked(api.getDayTimeline).mockResolvedValue(activityTimeline);

    render(<App />);
    fireEvent.click(
      await screen.findByRole("button", { name: "Edit activity: Museum visit" }),
    );
    fireEvent.change(
      within(screen.getByRole("region", { name: "Edit activity" })).getByLabelText(
        "Title",
      ),
      { target: { value: "Discard for New Trip" } },
    );
    fireEvent.click(screen.getByRole("button", { name: "New Trip" }));

    expect(screen.getByRole("region", { name: "Create a new trip" })).toBeInTheDocument();
    expect(screen.queryByRole("region", { name: "Edit activity" })).not.toBeInTheDocument();
    expect(api.updateItinerary).not.toHaveBeenCalled();
  });

  it("separates a committed activity update from refresh failure", async () => {
    await restoreWithSampleTrip();
    vi.mocked(api.getDayTimeline)
      .mockResolvedValueOnce(activityTimeline)
      .mockRejectedValueOnce({
        code: "STORAGE_FAILURE",
        message: "update refresh failed",
      })
      .mockResolvedValueOnce({
        ...activityTimeline,
        itineraries: [
          { ...sampleActivity, title: "Recovered edit", updated_at: "updated-2" },
        ],
      });

    render(<App />);
    fireEvent.click(
      await screen.findByRole("button", { name: "Edit activity: Museum visit" }),
    );
    const inspector = screen.getByRole("region", { name: "Edit activity" });
    fireEvent.change(within(inspector).getByLabelText("Title"), {
      target: { value: "Recovered edit" },
    });
    fireEvent.click(within(inspector).getByRole("button", { name: "Save" }));

    await waitFor(() =>
      expect(screen.getByRole("alert")).toHaveTextContent("update refresh failed"),
    );
    expect(screen.queryByRole("region", { name: "Edit activity" })).not.toBeInTheDocument();
    expect(api.updateItinerary).toHaveBeenCalledTimes(1);
    fireEvent.click(
      screen.getByRole("button", { name: /Day 1 · Sun · Apr 26/i }),
    );
    expect(await screen.findByText("Recovered edit")).toBeInTheDocument();
    expect(api.updateItinerary).toHaveBeenCalledTimes(1);
    expect(api.getDayTimeline).toHaveBeenCalledTimes(3);
  });

  it("does not refresh a stale activity update after Day navigation", async () => {
    await restoreWithSampleTrip();
    let resolveUpdate: (value: { itinerary_id: number }) => void = () => {};
    vi.mocked(api.updateItinerary).mockReturnValue(
      new Promise((resolve) => {
        resolveUpdate = resolve;
      }),
    );
    vi.mocked(api.getDayTimeline)
      .mockResolvedValueOnce(activityTimeline)
      .mockResolvedValueOnce({
        ...emptyTimeline,
        day_id: 11,
        day_number: 2,
        date: "2026-04-27",
        itineraries: [
          {
            ...sampleActivity,
            id: 30,
            day_number: 2,
            title: "Day 2 activity",
          },
        ],
      });

    render(<App />);
    fireEvent.click(
      await screen.findByRole("button", { name: "Edit activity: Museum visit" }),
    );
    const inspector = screen.getByRole("region", { name: "Edit activity" });
    fireEvent.change(within(inspector).getByLabelText("Title"), {
      target: { value: "Pending update" },
    });
    fireEvent.click(within(inspector).getByRole("button", { name: "Save" }));
    fireEvent.click(
      screen.getByRole("button", { name: /Day 2 · Mon · Apr 27/i }),
    );
    expect(await screen.findByText("Day 2 activity")).toBeInTheDocument();

    resolveUpdate({ itinerary_id: 20 });
    await waitFor(() => expect(api.updateItinerary).toHaveBeenCalledTimes(1));
    expect(api.getDayTimeline).toHaveBeenCalledTimes(2);
    expect(
      screen.getByRole("button", { name: /Day 2 · Mon · Apr 27/i }),
    ).toHaveAttribute("aria-pressed", "true");
    expect(screen.getByText("Day 2 activity")).toBeInTheDocument();
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

  it("adds an activity for the selected Day and refreshes only its timeline", async () => {
    await restoreWithSampleTrip();
    vi.mocked(api.getDayTimeline)
      .mockResolvedValueOnce(emptyTimeline)
      .mockResolvedValueOnce({
        ...emptyTimeline,
        itineraries: [
          {
            id: 20,
            trip_id: 1,
            day_number: 1,
            title: "  Sunset walk  ",
            start_time: "18:30",
            location: "  Naha waterfront  ",
            note: "  Bring a camera  ",
            sort_order: 1000,
            created_at: "t",
            updated_at: "t",
          },
        ],
      });

    render(<App />);
    await screen.findByRole("region", { name: "Plan" });
    fireEvent.click(screen.getByRole("button", { name: "Add activity" }));

    const form = screen.getByRole("region", { name: "Add activity" });
    fireEvent.change(within(form).getByLabelText("Title"), {
      target: { value: "  Sunset walk  " },
    });
    fireEvent.change(within(form).getByLabelText("Start time"), {
      target: { value: "18:30" },
    });
    fireEvent.change(within(form).getByLabelText("Location"), {
      target: { value: "  Naha waterfront  " },
    });
    fireEvent.change(within(form).getByLabelText("Note"), {
      target: { value: "  Bring a camera  " },
    });
    fireEvent.click(within(form).getByRole("button", { name: "Add activity" }));

    await waitFor(() =>
      expect(api.createItinerary).toHaveBeenCalledWith({
        trip_id: 1,
        day_number: 1,
        title: "  Sunset walk  ",
        start_time: "18:30",
        location: "  Naha waterfront  ",
        note: "  Bring a camera  ",
      }),
    );
    expect(await screen.findByText("Sunset walk", { exact: false })).toBeInTheDocument();
    expect(screen.queryByRole("region", { name: "Add activity" })).not.toBeInTheDocument();
    expect(api.getDayTimeline).toHaveBeenCalledTimes(2);
    expect(api.getTripDetail).toHaveBeenCalledTimes(1);
    expect(api.listTripSummaries).toHaveBeenCalledTimes(1);
    expect(
      screen.getByRole("button", { name: /Day 1 · Sun · Apr 26/i }),
    ).toHaveAttribute("aria-pressed", "true");
  });

  it("keeps quick-add input on write failure and does not refresh", async () => {
    await restoreWithSampleTrip();
    vi.mocked(api.createItinerary).mockRejectedValue({
      code: "ITINERARY_VALIDATION_FAILED",
      message: "Itinerary title must not be empty",
    });

    render(<App />);
    await screen.findByRole("region", { name: "Plan" });
    fireEvent.click(screen.getByRole("button", { name: "Add activity" }));
    const form = screen.getByRole("region", { name: "Add activity" });
    fireEvent.change(within(form).getByLabelText("Title"), {
      target: { value: "   " },
    });
    fireEvent.change(within(form).getByLabelText("Note"), {
      target: { value: "Keep this draft" },
    });
    fireEvent.click(within(form).getByRole("button", { name: "Add activity" }));

    await waitFor(() =>
      expect(screen.getByRole("alert")).toHaveTextContent(
        "Itinerary title must not be empty",
      ),
    );
    expect(within(form).getByLabelText("Title")).toHaveValue("   ");
    expect(within(form).getByLabelText("Note")).toHaveValue("Keep this draft");
    expect(api.getDayTimeline).toHaveBeenCalledTimes(1);
  });

  it("locks quick-add submission immediately", async () => {
    await restoreWithSampleTrip();
    let resolveCreate: (value: { itinerary_id: number }) => void = () => {};
    vi.mocked(api.createItinerary).mockReturnValue(
      new Promise((resolve) => {
        resolveCreate = resolve;
      }),
    );

    render(<App />);
    await screen.findByRole("region", { name: "Plan" });
    fireEvent.click(screen.getByRole("button", { name: "Add activity" }));
    const composer = screen.getByRole("region", { name: "Add activity" });
    fireEvent.change(within(composer).getByLabelText("Title"), {
      target: { value: "Coffee" },
    });
    const form = within(composer).getByRole("button", {
      name: "Add activity",
    }).closest("form");
    expect(form).not.toBeNull();
    fireEvent.submit(form!);
    fireEvent.submit(form!);

    expect(api.createItinerary).toHaveBeenCalledTimes(1);
    expect(within(composer).getByRole("button", { name: "Adding…" })).toBeDisabled();
    resolveCreate({ itinerary_id: 20 });
    await waitFor(() =>
      expect(screen.queryByRole("region", { name: "Add activity" })).not.toBeInTheDocument(),
    );
  });

  it("discards the quick-add draft on Day and Settings navigation", async () => {
    await restoreWithSampleTrip();
    vi.mocked(api.getDayTimeline)
      .mockResolvedValueOnce(emptyTimeline)
      .mockResolvedValueOnce({
        ...emptyTimeline,
        day_id: 11,
        day_number: 2,
        date: "2026-04-27",
      });

    render(<App />);
    await screen.findByRole("region", { name: "Plan" });
    fireEvent.click(screen.getByRole("button", { name: "Add activity" }));
    fireEvent.change(
      within(screen.getByRole("region", { name: "Add activity" })).getByLabelText(
        "Title",
      ),
      { target: { value: "Discard me" } },
    );
    fireEvent.click(
      screen.getByRole("button", { name: /Day 2 · Mon · Apr 27/i }),
    );
    await waitFor(() =>
      expect(screen.queryByRole("region", { name: "Add activity" })).not.toBeInTheDocument(),
    );

    fireEvent.click(screen.getByRole("button", { name: "Add activity" }));
    fireEvent.click(screen.getByRole("button", { name: /^settings$/i }));
    expect(screen.getByRole("region", { name: "Settings" })).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: /back to trips/i }));
    expect(screen.queryByRole("region", { name: "Add activity" })).not.toBeInTheDocument();
  });

  it("does not refresh a stale Day when navigation wins a pending write", async () => {
    await restoreWithSampleTrip();
    let resolveCreate: (value: { itinerary_id: number }) => void = () => {};
    vi.mocked(api.createItinerary).mockReturnValue(
      new Promise((resolve) => {
        resolveCreate = resolve;
      }),
    );
    vi.mocked(api.getDayTimeline)
      .mockResolvedValueOnce(emptyTimeline)
      .mockResolvedValueOnce({
        ...emptyTimeline,
        day_id: 11,
        day_number: 2,
        date: "2026-04-27",
        itineraries: [
          {
            id: 30,
            trip_id: 1,
            day_number: 2,
            title: "Day 2 activity",
            sort_order: 1000,
            created_at: "t",
            updated_at: "t",
          },
        ],
      });

    render(<App />);
    await screen.findByRole("region", { name: "Plan" });
    fireEvent.click(screen.getByRole("button", { name: "Add activity" }));
    const composer = screen.getByRole("region", { name: "Add activity" });
    fireEvent.change(within(composer).getByLabelText("Title"), {
      target: { value: "Day 1 pending" },
    });
    fireEvent.click(within(composer).getByRole("button", { name: "Add activity" }));
    fireEvent.click(
      screen.getByRole("button", { name: /Day 2 · Mon · Apr 27/i }),
    );
    await screen.findByText("Day 2 activity");

    resolveCreate({ itinerary_id: 20 });
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Add activity" })).not.toBeDisabled(),
    );
    expect(api.createItinerary).toHaveBeenCalledTimes(1);
    expect(api.getDayTimeline).toHaveBeenCalledTimes(2);
    expect(
      screen.getByRole("button", { name: /Day 2 · Mon · Apr 27/i }),
    ).toHaveAttribute("aria-pressed", "true");
    expect(screen.getByText("Day 2 activity")).toBeInTheDocument();
  });

  it("separates a successful write from refresh failure and retries the selected Day", async () => {
    await restoreWithSampleTrip();
    const refreshedTimeline = {
      ...emptyTimeline,
      itineraries: [
        {
          id: 20,
          trip_id: 1,
          day_number: 1,
          title: "Recovered activity",
          sort_order: 1000,
          created_at: "t",
          updated_at: "t",
        },
      ],
    };
    vi.mocked(api.getDayTimeline)
      .mockResolvedValueOnce(emptyTimeline)
      .mockRejectedValueOnce({
        code: "STORAGE_FAILURE",
        message: "refresh failed",
      })
      .mockResolvedValueOnce(refreshedTimeline);

    render(<App />);
    await screen.findByRole("region", { name: "Plan" });
    fireEvent.click(screen.getByRole("button", { name: "Add activity" }));
    const composer = screen.getByRole("region", { name: "Add activity" });
    fireEvent.change(within(composer).getByLabelText("Title"), {
      target: { value: "Recovered activity" },
    });
    fireEvent.click(within(composer).getByRole("button", { name: "Add activity" }));

    await waitFor(() =>
      expect(screen.getByRole("alert")).toHaveTextContent("refresh failed"),
    );
    expect(screen.queryByRole("region", { name: "Add activity" })).not.toBeInTheDocument();
    expect(api.createItinerary).toHaveBeenCalledTimes(1);

    fireEvent.click(
      screen.getByRole("button", { name: /Day 1 · Sun · Apr 26/i }),
    );
    expect(await screen.findByText("Recovered activity")).toBeInTheDocument();
    expect(api.createItinerary).toHaveBeenCalledTimes(1);
    expect(api.getDayTimeline).toHaveBeenCalledTimes(3);
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
