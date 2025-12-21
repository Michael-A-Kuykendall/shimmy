import { useState, useEffect } from "react";
import { SystemIcon, BlinkingLED, RotatingGear } from "./DecorativeElements";

const SystemStats = () => {
  const [cpuUsage, setCpuUsage] = useState(45);
  const [diskUsage, setDiskUsage] = useState(62);
  const [powerDraw, setPowerDraw] = useState(180);

  useEffect(() => {
    const interval = setInterval(() => {
      setCpuUsage((prev) => Math.max(20, Math.min(90, prev + Math.random() * 10 - 5)));
      setDiskUsage((prev) => Math.max(50, Math.min(80, prev + Math.random() * 2 - 1)));
      setPowerDraw((prev) => Math.max(150, Math.min(220, prev + Math.random() * 10 - 5)));
    }, 2000);

    return () => clearInterval(interval);
  }, []);

  return (
    <div className="space-y-2">
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-bold tracking-wider text-secondary flex items-center gap-2">
          <RotatingGear size="w-4 h-4" />
          PERFORMANCE
        </h3>
        <div className="flex gap-1">
          <BlinkingLED color="bg-primary" />
          <BlinkingLED color="bg-secondary" />
        </div>
      </div>
      
      <div className="grid grid-cols-3 gap-2">
        {/* CPU */}
        <div className="bg-muted/50 p-2 retro-border border-secondary/50 text-center pixel-shadow relative">
          <SystemIcon icon="cpu" active={cpuUsage > 60} />
          <div className="text-xs text-muted-foreground mt-1">CPU</div>
          <div className="text-sm font-mono font-bold text-foreground">
            {cpuUsage.toFixed(0)}%
          </div>
        </div>

        {/* Disk */}
        <div className="bg-muted/50 p-2 retro-border border-secondary/50 text-center pixel-shadow relative">
          <SystemIcon icon="disk" active={diskUsage > 70} />
          <div className="text-xs text-muted-foreground mt-1">DISK</div>
          <div className="text-sm font-mono font-bold text-foreground">
            {diskUsage.toFixed(0)}%
          </div>
        </div>

        {/* Power */}
        <div className="bg-muted/50 p-2 retro-border border-secondary/50 text-center pixel-shadow relative">
          <SystemIcon icon="power" active={powerDraw > 200} />
          <div className="text-xs text-muted-foreground mt-1">PWR</div>
          <div className="text-sm font-mono font-bold text-foreground">
            {powerDraw.toFixed(0)}W
          </div>
        </div>
      </div>

      {/* Additional System Info */}
      <div className="bg-muted/50 p-2 retro-border border-secondary/50 text-xs space-y-1 pixel-shadow relative">
        <div className="absolute top-1 right-1 w-2 h-2 border-t-2 border-r-2 border-secondary/50" />
        <div className="flex justify-between">
          <span className="text-muted-foreground">UPTIME:</span>
          <span className="font-mono text-foreground">02:34:12</span>
        </div>
        <div className="flex justify-between">
          <span className="text-muted-foreground">REQUESTS:</span>
          <span className="font-mono text-foreground">1,247</span>
        </div>
        <div className="flex justify-between">
          <span className="text-muted-foreground">LATENCY:</span>
          <span className="font-mono text-accent">23ms</span>
        </div>
      </div>
    </div>
  );
};

export default SystemStats;
