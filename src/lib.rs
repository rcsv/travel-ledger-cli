//! Caglla.Travel CLI core library.
//!
//! Read facade and DB boundary for the CLI binary and future Desktop (Tauri) crates.
//! Terminal presentation remains in the binary; this crate exposes use cases and DTOs.

mod analysis;
mod checklist;
mod cli;
mod cli_run;
mod commands;
mod config;
mod day;
mod domain;
mod estimate;
mod expense;
mod geo;
mod io;
mod itinerary;
mod money;
mod note;
mod output;
mod participant;
mod proposal;
mod receipt;
mod reservation;
mod services;
mod storage;
mod summary;
mod trip;
mod trip_metadata;

pub use services::{
    get_day_timeline, get_trip_detail, list_trip_summaries, DayDetail, DaySummary, ItineraryDetail,
    ReadServiceErrorCode, ServiceError, TripDetail, TripSummary,
};

/// Opens a SQLite database at `path` and ensures schema/migrations are applied.
pub fn open_db(path: &str) -> anyhow::Result<rusqlite::Connection> {
    storage::db::open_db_at(path)
}

/// Runs the CLI (used by the `travel-ledger-cli` binary).
pub fn run() -> anyhow::Result<()> {
    cli_run::run()
}
