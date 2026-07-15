import { useEffect, useState } from "react";
import { getVersion } from "@tauri-apps/api/app";

import { databaseFileName } from "../types";

interface SettingsPanelProps {
  databasePath: string;
  onChangeDatabase: () => void;
  onForgetDatabase: () => void;
  onBackToTrips: () => void;
}

export function SettingsPanel({
  databasePath,
  onChangeDatabase,
  onForgetDatabase,
  onBackToTrips,
}: SettingsPanelProps) {
  const [appVersion, setAppVersion] = useState<string>("…");

  useEffect(() => {
    let cancelled = false;
    void getVersion()
      .then((version) => {
        if (!cancelled) {
          setAppVersion(version);
        }
      })
      .catch(() => {
        if (!cancelled) {
          setAppVersion("unavailable");
        }
      });
    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <section className="settings-panel" aria-label="Settings">
      <header className="settings-header">
        <div>
          <h2>Settings</h2>
          <p className="settings-lede">
            Database and app information for this developer preview.
          </p>
        </div>
        <button
          type="button"
          className="secondary-button"
          onClick={onBackToTrips}
        >
          Back to trips
        </button>
      </header>

      <section className="settings-section" aria-labelledby="settings-database">
        <h3 id="settings-database">Database</h3>
        <p className="settings-status">Access: local Trip creation</p>
        <dl className="settings-meta">
          <div className="meta-row">
            <dt>File</dt>
            <dd>{databaseFileName(databasePath)}</dd>
          </div>
          <div className="meta-row settings-path-row">
            <dt>Path</dt>
            <dd className="settings-path" title={databasePath}>
              {databasePath}
            </dd>
          </div>
        </dl>
        <div className="settings-actions">
          <button
            type="button"
            className="secondary-button"
            onClick={onChangeDatabase}
          >
            Change Database
          </button>
          <button
            type="button"
            className="secondary-button"
            onClick={onForgetDatabase}
            title="Clears the remembered path only. Does not delete the database file."
          >
            Forget Database
          </button>
        </div>
        <p className="settings-note">
          Forget clears only the remembered path and in-app selection. Your
          SQLite database file is not deleted.
        </p>
      </section>

      <section className="settings-section" aria-labelledby="settings-about">
        <h3 id="settings-about">About</h3>
        <dl className="settings-meta">
          <div className="meta-row">
            <dt>App</dt>
            <dd>Travel Ledger Desktop</dd>
          </div>
          <div className="meta-row">
            <dt>Status</dt>
            <dd>Developer preview</dd>
          </div>
          <div className="meta-row">
            <dt>Version</dt>
            <dd>{appVersion}</dd>
          </div>
        </dl>
      </section>
    </section>
  );
}
