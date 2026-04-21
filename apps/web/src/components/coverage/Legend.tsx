"use client";

import type { LegendEntry } from "@/lib/types";

interface Props {
  legend: LegendEntry[];
  rxSensitivity?: number;
}

function sanitizeRgba(rgba: [number, number, number, number]): string {
  const [r, g, b, a] = rgba;
  return `rgba(${Math.round(r)},${Math.round(g)},${Math.round(b)},${a / 255})`;
}

export default function Legend({ legend, rxSensitivity }: Props) {
  const sorted = [...legend].sort((a, b) => b.threshold_dbm - a.threshold_dbm);

  return (
    <div className="space-y-1 text-xs">
      {sorted.map((entry, i) => {
        return (
          <div key={i} className="flex items-center gap-2">
            <span
              className="inline-block w-4 h-4 rounded-sm flex-shrink-0 bg-[var(--legend-color)]"
              style={{ "--legend-color": sanitizeRgba(entry.rgba) } as React.CSSProperties}
            />
            <span className="text-gray-300">≥ {entry.threshold_dbm} dBm</span>
            <span className="text-gray-500 ml-auto">{entry.label}</span>
          </div>
        );
      })}
      {rxSensitivity !== undefined && (
        <div className="text-gray-500 pt-1 border-t border-white/10">
          RX sensitivity: {rxSensitivity} dBm
        </div>
      )}
    </div>
  );
}
