import { useState } from "react";
import { Settings, ChevronUp } from "lucide-react";
import { Popover, PopoverContent, PopoverTrigger } from "./ui/popover";
import { Checkbox } from "./ui/checkbox";
import { Button } from "./ui/button";

export interface MetricOption {
  id: string;
  label: string;
  unit: string;
  getValue: (metrics: any) => number;
  getMax: (metrics: any) => number;
  color: string;
  formatValue?: (value: number) => string;
  minLabel?: string;
  maxLabel?: string;
  showMarkLabels?: boolean;
}

export const AVAILABLE_METRICS: MetricOption[] = [
  {
    id: "cpu",
    label: "CPU USAGE",
    unit: "%",
    getValue: (m) => Math.round(m?.cpu_usage_percent || 0),
    getMax: () => 100,
    color: "text-accent",
    formatValue: (v) => `${v}`,
    minLabel: "0%",
    maxLabel: "100%",
    showMarkLabels: true,
  },
  {
    id: "ram",
    label: "RAM",
    unit: "GB",
    getValue: (m) => m ? m.memory_used_bytes / (1024 * 1024 * 1024) : 0,
    getMax: (m) => m ? m.memory_total_bytes / (1024 * 1024 * 1024) : 8,
    color: "text-secondary",
    formatValue: (v) => v.toFixed(1),
    showMarkLabels: true,
  },
  {
    id: "tokens",
    label: "SESSION TOKENS",
    unit: "TOKENS",
    getValue: (m) => m?.current_session_tokens || 0,
    getMax: () => 8192,
    color: "text-primary",
    formatValue: (v) => v >= 1000 ? `${(v / 1000).toFixed(1)}K` : `${v}`,
    minLabel: "0",
    maxLabel: "8K",
    showMarkLabels: true,
  },
  {
    id: "tps",
    label: "TOKENS/SEC",
    unit: "T/S",
    getValue: (m) => m?.tokens_per_second || 0,
    getMax: () => 50,
    color: "text-accent",
    formatValue: (v) => v.toFixed(1),
    minLabel: "0",
    maxLabel: "50",
    showMarkLabels: true,
  },
  {
    id: "gpu",
    label: "GPU USAGE",
    unit: "%",
    getValue: (m) => Math.round(m?.gpu_usage_percent || 0),
    getMax: () => 100,
    color: "text-primary",
    formatValue: (v) => `${v}`,
    minLabel: "0%",
    maxLabel: "100%",
    showMarkLabels: true,
  },
  {
    id: "vram",
    label: "VRAM",
    unit: "GB",
    getValue: (m) => m?.gpu_memory_used_bytes ? m.gpu_memory_used_bytes / (1024 * 1024 * 1024) : 0,
    getMax: (m) => m?.gpu_memory_total_bytes ? m.gpu_memory_total_bytes / (1024 * 1024 * 1024) : 8,
    color: "text-secondary",
    formatValue: (v) => v.toFixed(1),
    showMarkLabels: true,
  },
  {
    id: "context",
    label: "CONTEXT SIZE",
    unit: "TOKENS",
    getValue: (m) => m?.context_size_current || 0,
    getMax: (m) => m?.context_size_max || 4096,
    color: "text-accent",
    formatValue: (v) => v >= 1000 ? `${(v / 1000).toFixed(1)}K` : `${v}`,
    showMarkLabels: true,
  },
  {
    id: "batch",
    label: "BATCH SIZE",
    unit: "SIZE",
    getValue: (m) => m?.batch_size || 0,
    getMax: () => 1024,
    color: "text-primary",
    formatValue: (v) => `${v}`,
    minLabel: "0",
    maxLabel: "1024",
    showMarkLabels: true,
  },
  {
    id: "uptime",
    label: "UPTIME",
    unit: "TIME",
    getValue: (m) => (m?.uptime_seconds || 0),
    getMax: () => 86400, // 24 hours in seconds
    color: "text-secondary",
    formatValue: (v) => {
      const hours = Math.floor(v / 3600);
      const minutes = Math.floor((v % 3600) / 60);
      return `${hours}:${minutes.toString().padStart(2, '0')}`;
    },
    minLabel: "0h",
    maxLabel: "24h",
    showMarkLabels: true,
  },
  {
    id: "requests",
    label: "REQUESTS",
    unit: "TOTAL",
    getValue: (m) => m?.requests_total || 0,
    getMax: () => 1000,
    color: "text-accent",
    formatValue: (v) => v >= 1000 ? `${(v / 1000).toFixed(1)}K` : `${v}`,
    minLabel: "0",
    maxLabel: "1K",
    showMarkLabels: true,
  },
  {
    id: "errors",
    label: "ERRORS",
    unit: "COUNT",
    getValue: (m) => m?.generation_errors || 0,
    getMax: () => 100,
    color: "text-destructive",
    formatValue: (v) => `${v}`,
    minLabel: "0",
    maxLabel: "100",
    showMarkLabels: true,
  },
  {
    id: "avg_tps",
    label: "AVG TOKENS/S",
    unit: "T/S",
    getValue: (m) => m?.rolling_avg_tps || 0,
    getMax: () => 50,
    color: "text-primary",
    formatValue: (v) => v.toFixed(1),
    minLabel: "0",
    maxLabel: "50",
    showMarkLabels: true,
  },
];

interface MetricsSelectorProps {
  selectedMetrics: string[];
  onMetricsChange: (metrics: string[]) => void;
}

export const MetricsSelector = ({ selectedMetrics, onMetricsChange }: MetricsSelectorProps) => {
  const [open, setOpen] = useState(false);

  const handleToggle = (metricId: string) => {
    if (selectedMetrics.includes(metricId)) {
      onMetricsChange(selectedMetrics.filter(id => id !== metricId));
    } else if (selectedMetrics.length < 4) {
      onMetricsChange([...selectedMetrics, metricId]);
    }
  };

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button
          variant="outline"
          className="w-full chunky-button bg-muted/50 hover:bg-muted border-accent/50 gap-2"
        >
          <Settings className="h-4 w-4" />
          <span className="text-xs font-bold tracking-widest flex-1 text-left">
            CONFIGURE METRICS ({selectedMetrics.length}/4)
          </span>
          <ChevronUp className={`h-4 w-4 transition-transform ${open ? "" : "rotate-180"}`} />
        </Button>
      </PopoverTrigger>
      <PopoverContent 
        className="w-80 chunky-border border-accent bg-card p-4 pixel-shadow"
        align="end"
        side="top"
      >
        <div className="space-y-3">
          <div className="border-b-4 border-accent/50 pb-2">
            <h3 className="text-sm font-bold tracking-wider text-accent">SELECT METRICS (MAX 4)</h3>
          </div>
          <div className="grid grid-cols-2 gap-3 max-h-64 overflow-y-auto">
            {AVAILABLE_METRICS.map((metric) => (
              <div
                key={metric.id}
                className="flex items-center gap-2 p-2 chunky-inset bg-muted/30 hover:bg-muted/50 cursor-pointer transition-colors"
                onClick={() => handleToggle(metric.id)}
              >
                <Checkbox
                  checked={selectedMetrics.includes(metric.id)}
                  disabled={!selectedMetrics.includes(metric.id) && selectedMetrics.length >= 4}
                  className="pointer-events-none"
                />
                <span className="text-xs font-mono font-bold">
                  {metric.label}
                </span>
              </div>
            ))}
          </div>
        </div>
      </PopoverContent>
    </Popover>
  );
};
