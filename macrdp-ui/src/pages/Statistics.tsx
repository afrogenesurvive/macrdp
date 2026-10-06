import { useState, useEffect, useMemo } from "react";
import { ChevronLeft, ChevronRight } from "lucide-react";
import { api } from "../lib/ipc";
import { formatBytes, formatDuration } from "../lib/format";
import MetricCard from "@/components/MetricCard";
import { ChartPanel } from "@/components/ChartPanel";
import { BarChart } from "@/components/BarChart";
import EmptyState from "@/components/EmptyState";
import type { ConnectionHistory, TrafficStats } from "../lib/types";
import { useI18n } from "../i18n";

const PAGE_SIZE = 20;
type TimeRange = 7 | 30;

function Statistics() {
  const { t, locale } = useI18n();
  const [days, setDays] = useState<TimeRange>(7);

  // Traffic stats
  const [trafficStats, setTrafficStats] = useState<TrafficStats[]>([]);

  // Connection history
  const [history, setHistory] = useState<ConnectionHistory[]>([]);
  const [page, setPage] = useState(0);
  const [hasMore, setHasMore] = useState(true);

  useEffect(() => {
    api.getTrafficStats(days).then(setTrafficStats).catch(console.error);
  }, [days]);

  useEffect(() => {
    fetchHistory(0);
  }, []);

  const fetchHistory = async (p: number) => {
    try {
      const data = await api.getConnectionHistory(PAGE_SIZE, p * PAGE_SIZE);
      setHistory(data);
      setPage(p);
      setHasMore(data.length === PAGE_SIZE);
    } catch (err) {
      console.error("Failed to fetch connection history:", err);
    }
  };

  const totalConnections = useMemo(
    () => trafficStats.reduce((sum, d) => sum + d.connection_count, 0),
    [trafficStats],
  );

  const totalTraffic = useMemo(
    () => trafficStats.reduce((sum, d) => sum + d.bytes_sent, 0),
    [trafficStats],
  );

  const avgSessionDuration = useMemo(() => {
    if (history.length === 0) return null;
    const total = history.reduce((sum, c) => sum + c.duration_secs, 0);
    return Math.round(total / history.length);
  }, [history]);

  const chartData = useMemo(
    () =>
      trafficStats.map((d) => {
        const dt = new Date(d.date);
        const label = dt.toLocaleDateString(locale, {
          month: "2-digit",
          day: "2-digit",
        });
        return { label, value: d.bytes_sent };
      }),
    [trafficStats, locale],
  );

  const formatDateTime = (dateStr: string) => {
    try {
      const d = new Date(dateStr);
      return d.toLocaleString(locale, {
        month: "2-digit",
        day: "2-digit",
        hour: "2-digit",
        minute: "2-digit",
        hour12: false,
      });
    } catch {
      return dateStr;
    }
  };

  const pillBase =
    "text-[10px] px-2 py-0.5 rounded-[4px] cursor-pointer transition-colors";
  const pillActive =
    "bg-accent/15 border border-accent/30 text-accent";
  const pillDefault =
    "bg-card border border-border-subtle text-text-muted";

  return (
    <div className="flex flex-col gap-3 p-4 overflow-y-auto">
      {/* 1. Header + time range toggle */}
      <div className="flex items-center justify-between">
        <h1 className="text-base font-semibold">{t("stats.title")}</h1>
        <div className="flex gap-1">
          <button
            className={`${pillBase} ${days === 7 ? pillActive : pillDefault}`}
            onClick={() => setDays(7)}
          >
            {t("stats.days7")}
          </button>
          <button
            className={`${pillBase} ${days === 30 ? pillActive : pillDefault}`}
            onClick={() => setDays(30)}
          >
            {t("stats.days30")}
          </button>
        </div>
      </div>

      {/* 2. Summary cards */}
      <div className="flex gap-2">
        <MetricCard
          value={totalConnections}
          label={t("stats.totalConnections")}
          color="blue"
        />
        <MetricCard
          value={totalTraffic}
          label={t("stats.totalTraffic")}
          color="green"
          formatter={formatBytes}
        />
        <MetricCard
          value={avgSessionDuration}
          label={t("stats.avgSession")}
          color="orange"
          formatter={formatDuration}
        />
      </div>

      {/* 3. Traffic trend chart */}
      <ChartPanel
        title={t("stats.trafficTrend")}
        empty={chartData.length === 0}
        emptyText={t("stats.noTrafficData")}
      >
        <BarChart data={chartData} color="var(--color-accent)" />
      </ChartPanel>

      {/* 4. Connection history table */}
      <div className="bg-card rounded-[8px] p-3">
        {history.length === 0 && page === 0 ? (
          <EmptyState message={t("stats.noHistory")} />
        ) : (
          <>
            <table className="w-full">
              <thead>
                <tr className="text-[10px] text-text-muted font-medium">
                  <th className="text-left pb-2">{t("stats.user")}</th>
                  <th className="text-left pb-2">{t("stats.time")}</th>
                  <th className="text-left pb-2">{t("stats.duration")}</th>
                  <th className="text-right pb-2">{t("stats.traffic")}</th>
                </tr>
              </thead>
              <tbody>
                {history.map((conn) => (
                  <tr
                    key={conn.id}
                    className="text-[11px] text-text-secondary border-b border-border"
                  >
                    <td className="py-1.5">{conn.client_name || conn.client_ip}</td>
                    <td className="py-1.5">{formatDateTime(conn.connected_at)}</td>
                    <td className="py-1.5">{formatDuration(conn.duration_secs)}</td>
                    <td className="py-1.5 text-right">{formatBytes(conn.bytes_total)}</td>
                  </tr>
                ))}
              </tbody>
            </table>

            {/* Pagination */}
            <div className="mt-2 flex items-center justify-between">
              <span className="text-[10px] text-text-muted">
                {t("stats.page", { n: page + 1 })}
              </span>
              <div className="flex gap-1.5">
                <button
                  className="text-[10px] text-text-muted disabled:opacity-30 flex items-center gap-0.5"
                  disabled={page === 0}
                  onClick={() => fetchHistory(page - 1)}
                >
                  <ChevronLeft className="h-3 w-3" />
                  {t("stats.prev")}
                </button>
                <button
                  className="text-[10px] text-text-muted disabled:opacity-30 flex items-center gap-0.5"
                  disabled={!hasMore}
                  onClick={() => fetchHistory(page + 1)}
                >
                  {t("stats.next")}
                  <ChevronRight className="h-3 w-3" />
                </button>
              </div>
            </div>
          </>
        )}
      </div>
    </div>
  );
}

export default Statistics;
