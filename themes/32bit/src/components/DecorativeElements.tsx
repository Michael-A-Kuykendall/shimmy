import { Cpu, Zap, Activity, HardDrive } from "lucide-react";

export const BlinkingLED = ({ color = "bg-secondary" }: { color?: string }) => (
  <div className={`w-3 h-3 ${color} rounded-sm led-indicator pixel-shadow`} />
);

export const RotatingGear = ({ size = "w-6 h-6" }: { size?: string }) => (
  <div className={`${size} text-primary/60 rotating-gear`}>
    <Cpu className="w-full h-full" />
  </div>
);

export const EnergyBar = ({ value = 75 }: { value?: number }) => (
  <div className="flex gap-1 items-center">
    {Array.from({ length: 8 }).map((_, i) => (
      <div
        key={i}
        className={`w-1 h-4 ${
          i < (value / 100) * 8 ? "bg-secondary energy-pulse" : "bg-muted/30"
        } retro-border border-secondary/50`}
      />
    ))}
  </div>
);

export const ProcessingIndicator = () => (
  <div className="flex items-center gap-2 text-xs text-accent">
    <Activity className="w-4 h-4 animate-pulse" />
    <span className="font-mono tracking-wider">PROCESSING</span>
  </div>
);

export const SystemIcon = ({ 
  icon, 
  active = false 
}: { 
  icon: "cpu" | "disk" | "power"; 
  active?: boolean;
}) => {
  const icons = {
    cpu: Cpu,
    disk: HardDrive,
    power: Zap,
  };
  const Icon = icons[icon];
  
  return (
    <div className={`w-8 h-8 retro-border ${
      active 
        ? "border-accent bg-accent/20 text-accent" 
        : "border-muted-foreground/30 bg-muted/20 text-muted-foreground"
    } flex items-center justify-center pixel-shadow`}>
      <Icon className="w-5 h-5" />
    </div>
  );
};

export const DecorativeCorner = () => (
  <div className="absolute top-0 left-0 w-4 h-4 border-t-4 border-l-4 border-secondary" />
);