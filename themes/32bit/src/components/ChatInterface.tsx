import MessageList from "./MessageList";
import InputBox from "./InputBox";
import { Card } from "./ui/card";
import { BlinkingLED } from "./DecorativeElements";
import { useShimmy } from "@/hooks/useShimmy";

export interface Message {
  id: string;
  role: "user" | "assistant";
  content: string;
  timestamp: Date;
}

const ChatInterface = () => {
  const { messages, sendMessage, isConnected, isGenerating } = useShimmy();

  return (
    <Card className="h-full flex flex-col chunky-border border-primary bg-card/95 backdrop-blur-sm pixel-shadow relative overflow-hidden">
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b-8 border-primary/50 bg-primary/20 chunky-inset">
        <div className="flex items-center gap-3">
          <h2 className="text-2xl font-bold tracking-widest text-secondary glow-text" style={{ textShadow: '3px 3px 0 rgba(0,0,0,0.5)' }}>SHIMMY</h2>
          <div className="flex gap-1">
            <div className="w-12 h-3 bg-secondary chunky-inset" />
            <div className="w-12 h-3 bg-secondary chunky-inset" />
            <div className="w-12 h-3 bg-secondary chunky-inset" />
          </div>
        </div>
        <div className="flex items-center gap-2">
          <span className="text-xs font-mono text-muted-foreground">
            {isConnected ? 'CONNECTED' : 'DISCONNECTED'}
          </span>
          <BlinkingLED color={isConnected ? 'bg-secondary' : 'bg-destructive'} />
        </div>
      </div>
      
      {/* Chat Messages */}
      <div className="flex-1 overflow-hidden">
        <MessageList messages={messages} />
      </div>

      {/* Input Box */}
      <InputBox onSend={sendMessage} disabled={!isConnected || isGenerating} />
    </Card>
  );
};

export default ChatInterface;
