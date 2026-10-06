import { useState } from "react";
import { Monitor } from "lucide-react";
import { api } from "../lib/ipc";
import { useI18n } from "../i18n";

function About() {
  const { t } = useI18n();
  const [updateStatus, setUpdateStatus] = useState<
    "idle" | "checking" | "latest" | "available" | "error"
  >("idle");
  const [updateInfo, setUpdateInfo] = useState<{
    version?: string;
    url?: string;
  }>({});

  const handleCheckUpdate = async () => {
    setUpdateStatus("checking");
    try {
      const result = await api.checkForUpdates();
      if (result.available) {
        setUpdateStatus("available");
        setUpdateInfo({ version: result.version, url: result.url });
      } else {
        setUpdateStatus("latest");
      }
    } catch (err) {
      console.error("Failed to check for updates:", err);
      setUpdateStatus("error");
    }
  };

  const updateLabel = () => {
    switch (updateStatus) {
      case "checking":
        return t("about.checking");
      case "latest":
        return t("about.latest");
      case "available":
        return t("about.available", { v: updateInfo.version ?? "" });
      case "error":
        return t("about.checkFailed");
      default:
        return t("about.check");
    }
  };

  return (
    <div className="flex h-full flex-col items-center justify-center p-4">
      <div className="w-full max-w-sm rounded-[10px] bg-card p-6 text-center">
        {/* App icon */}
        <div className="mx-auto mb-3 flex h-16 w-16 items-center justify-center rounded-2xl bg-accent/10">
          <Monitor size={32} className="text-accent" />
        </div>

        {/* App name */}
        <div className="text-lg font-semibold text-text">MacRDP</div>

        {/* Version */}
        <div className="mt-0.5 text-xs text-text-muted">{t("about.version", { v: "1.0.0" })}</div>

        {/* Separator */}
        <div className="my-3 border-t border-border" />

        {/* Tech stack */}
        <div className="text-[11px] text-text-muted">
          IronRDP &middot; OpenH264 &middot; ScreenCaptureKit
        </div>

        {/* Links row */}
        <div className="mt-3 flex items-center justify-center gap-3">
          <a
            href="https://github.com/aspect-build/macrdp"
            target="_blank"
            rel="noopener noreferrer"
            className="text-xs text-accent hover:underline"
          >
            GitHub
          </a>
          <span className="text-xs text-text-muted">MIT License</span>
          <button
            onClick={handleCheckUpdate}
            disabled={updateStatus === "checking"}
            className="text-xs text-accent hover:underline disabled:opacity-50"
          >
            {updateLabel()}
          </button>
        </div>

        {/* Update download link */}
        {updateStatus === "available" && updateInfo.url && (
          <div className="mt-2">
            <a
              href={updateInfo.url}
              target="_blank"
              rel="noopener noreferrer"
              className="text-xs text-accent hover:underline"
            >
              {t("about.download")}
            </a>
          </div>
        )}
      </div>
    </div>
  );
}

export default About;
