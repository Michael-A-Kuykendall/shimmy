import React, { useState, useEffect, useRef } from 'react'

interface Message {
  id: string
  role: 'user' | 'assistant'
  content: string
  timestamp: Date
  isStreaming?: boolean
}

interface ChatProps {
  socket: WebSocket | null
  selectedModel: string
  onModelChange: () => void
}

function Chat({ socket, selectedModel, onModelChange }: ChatProps) {
  const [messages, setMessages] = useState<Message[]>([])
  const [input, setInput] = useState('')
  const [isGenerating, setIsGenerating] = useState(false)
  const [currentStreamingId, setCurrentStreamingId] = useState<string | null>(null)
  
  const messagesEndRef = useRef<HTMLDivElement>(null)
  const inputRef = useRef<HTMLTextAreaElement>(null)
  
  // Auto-scroll to bottom when new messages arrive
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [messages])
  
  // Listen for WebSocket messages
  useEffect(() => {
    const handleMessage = (event: CustomEvent) => {
      const message = event.detail
      
      // Handle canonical streaming protocol: {token: "..."} and {done: true}
      if (message.token && currentStreamingId) {
        // Streaming token
        setMessages(prev => prev.map(msg => 
          msg.id === currentStreamingId 
            ? { ...msg, content: msg.content + message.token }
            : msg
        ))
      } else if (message.done) {
        // Generation complete
        setIsGenerating(false)
        setCurrentStreamingId(null)
        
        // Mark the streaming message as complete
        setMessages(prev => prev.map(msg => 
          msg.isStreaming 
            ? { ...msg, isStreaming: false }
            : msg
        ))
        
        // Focus input for next message
        setTimeout(() => inputRef.current?.focus(), 100)
      } else {
        // Handle other message types
        switch (message.type) {
          case 'error':
            setIsGenerating(false)
            setCurrentStreamingId(null)
            
            // Add error message
            const errorMessage: Message = {
              id: Date.now().toString(),
              role: 'assistant',
              content: `❌ Error: ${message.error || 'Unknown error occurred'}`,
              timestamp: new Date()
            }
            setMessages(prev => [...prev, errorMessage])
            break
        }
      }
    }
    
    window.addEventListener('shimmy-websocket-message', handleMessage as EventListener)
    
    return () => {
      window.removeEventListener('shimmy-websocket-message', handleMessage as EventListener)
    }
  }, [currentStreamingId])
  
  const sendMessage = () => {
    if (!socket || !input.trim() || isGenerating) return
    
    const userMessage: Message = {
      id: Date.now().toString(),
      role: 'user',
      content: input.trim(),
      timestamp: new Date()
    }
    
    const assistantMessageId = (Date.now() + 1).toString()
    const assistantMessage: Message = {
      id: assistantMessageId,
      role: 'assistant',
      content: '',
      timestamp: new Date(),
      isStreaming: true
    }
    
    setMessages(prev => [...prev, userMessage, assistantMessage])
    setCurrentStreamingId(assistantMessageId)
    setIsGenerating(true)
    setInput('')
    
    // Send chat request
    socket.send(JSON.stringify({
      type: 'chat_request',
      message: input.trim(),
      model: selectedModel
    }))
  }
  
  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      sendMessage()
    }
  }
  
  const clearChat = () => {
    setMessages([])
    setIsGenerating(false)
    setCurrentStreamingId(null)
  }
  
  const formatTimestamp = (date: Date) => {
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
  }
  
  if (!socket) {
    return (
      <div className="text-center py-12">
        <div className="text-gray-500">
          <div className="text-4xl mb-4">🔌</div>
          <p>Waiting for WebSocket connection...</p>
        </div>
      </div>
    )
  }
  
  return (
    <div className="h-full flex flex-col bg-gray-800 rounded-lg">
      {/* Header */}
      <div className="border-b border-gray-700 p-4 flex items-center justify-between">
        <div>
          <h2 className="text-xl font-semibold text-blue-400">Chat</h2>
          <p className="text-sm text-gray-400">
            Model: {selectedModel}
          </p>
        </div>
        
        <div className="flex items-center space-x-2">
          <button
            onClick={clearChat}
            className="px-3 py-1 text-sm bg-gray-600 hover:bg-gray-500 rounded text-white transition-colors"
          >
            🗑️ Clear
          </button>
          
          <button
            onClick={onModelChange}
            className="px-3 py-1 text-sm bg-blue-600 hover:bg-blue-700 rounded text-white transition-colors"
          >
            🔄 Change Model
          </button>
        </div>
      </div>
      
      {/* Messages */}
      <div className="flex-1 overflow-y-auto p-4 space-y-4">
        {messages.length === 0 && (
          <div className="text-center py-12">
            <div className="text-6xl mb-4">💬</div>
            <h3 className="text-xl mb-2 text-gray-400">Start a conversation</h3>
            <p className="text-gray-500">
              Type a message below to begin chatting with {selectedModel}
            </p>
          </div>
        )}
        
        {messages.map((message) => (
          <div
            key={message.id}
            className={`flex ${message.role === 'user' ? 'justify-end' : 'justify-start'}`}
          >
            <div
              className={`
                max-w-3xl rounded-lg px-4 py-3 text-sm
                ${message.role === 'user'
                  ? 'bg-blue-600 text-white ml-12'
                  : 'bg-gray-700 text-gray-100 mr-12'
                }
                ${message.isStreaming ? 'animate-pulse' : ''}
              `}
            >
              <div className="flex items-start space-x-3">
                <div className="text-lg">
                  {message.role === 'user' ? '👤' : '🤖'}
                </div>
                
                <div className="flex-1">
                  <div className="whitespace-pre-wrap break-words">
                    {message.content}
                    {message.isStreaming && (
                      <span className="inline-block w-2 h-4 bg-gray-400 ml-1 animate-pulse">|</span>
                    )}
                  </div>
                  
                  <div className={`text-xs mt-2 opacity-70 ${
                    message.role === 'user' ? 'text-blue-200' : 'text-gray-400'
                  }`}>
                    {formatTimestamp(message.timestamp)}
                    {message.isStreaming && (
                      <span className="ml-2">✏️ Generating...</span>
                    )}
                  </div>
                </div>
              </div>
            </div>
          </div>
        ))}
        
        <div ref={messagesEndRef} />
      </div>
      
      {/* Input */}
      <div className="border-t border-gray-700 p-4">
        <div className="flex space-x-4">
          <div className="flex-1">
            <textarea
              ref={inputRef}
              value={input}
              onChange={(e) => setInput(e.target.value)}
              onKeyPress={handleKeyPress}
              placeholder={isGenerating ? "Generating response..." : "Type your message..."}
              disabled={isGenerating}
              rows={3}
              className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:border-blue-500 focus:outline-none resize-none disabled:opacity-50"
            />
            
            <div className="flex items-center justify-between mt-2 text-xs text-gray-400">
              <span>
                Press Enter to send, Shift+Enter for new line
              </span>
              
              <span>
                {input.length} characters
              </span>
            </div>
          </div>
          
          <button
            onClick={sendMessage}
            disabled={!input.trim() || isGenerating}
            className="px-6 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-600 disabled:cursor-not-allowed rounded-lg text-white font-medium transition-colors self-start"
          >
            {isGenerating ? (
              <span className="flex items-center">
                <div className="animate-spin rounded-full h-4 w-4 border-2 border-white border-t-transparent mr-2"></div>
                Sending...
              </span>
            ) : (
              '📤 Send'
            )}
          </button>
        </div>
      </div>
    </div>
  )
}

export default Chat