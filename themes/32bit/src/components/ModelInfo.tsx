import { BlinkingLED, RotatingGear } from "./DecorativeElements";
import { useShimmy } from "@/hooks/useShimmy";

const ModelInfo = () => {
  const { models, isConnected } = useShimmy();
  const model = models[0];

  return (
    <div className="space-y-2 relative">
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-bold tracking-wider text-secondary flex items-center gap-2">
          <RotatingGear size="w-5 h-5" />
          MODEL INFO
        </h3>
        <BlinkingLED color="bg-accent" />
      </div>
      <div className="bg-muted/50 p-3 retro-border border-secondary/50 space-y-2 pixel-shadow relative">
        {/* Decorative corner accents */}
        <div className="absolute top-1 left-1 w-2 h-2 border-t-2 border-l-2 border-secondary/50" />
        <div className="absolute top-1 right-1 w-2 h-2 border-t-2 border-r-2 border-secondary/50" />
        <div className="absolute bottom-1 left-1 w-2 h-2 border-b-2 border-l-2 border-secondary/50" />
        <div className="absolute bottom-1 right-1 w-2 h-2 border-b-2 border-r-2 border-secondary/50" />
        
        <div className="flex justify-between text-xs">
          <span className="text-muted-foreground">NAME:</span>
          <span className="font-mono text-foreground">{model?.name || 'NO MODEL'}</span>
        </div>
        <div className="flex justify-between text-xs">
          <span className="text-muted-foreground">PARAMS:</span>
          <span className="font-mono text-foreground">{model?.parameter_count || 'N/A'}</span>
        </div>
        <div className="flex justify-between text-xs">
          <span className="text-muted-foreground">STATUS:</span>
          <span className={`font-mono flex items-center gap-1 ${isConnected ? 'text-accent' : 'text-destructive'}`}>
            <BlinkingLED color={isConnected ? 'bg-accent' : 'bg-destructive'} />
            {isConnected ? 'ONLINE' : 'OFFLINE'}
          </span>
        </div>
        <div className="flex justify-between text-xs">
          <span className="text-muted-foreground">CONTEXT:</span>
          <span className="font-mono text-foreground">8192 TOKENS</span>
        </div>
      </div>
    </div>
  );
};

export default ModelInfo;
