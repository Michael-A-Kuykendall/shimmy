# test-theme-2

A Shimmy frontend theme generated from schema-driven architecture.

## Overview

This theme was automatically generated using the Shimmy Theme Generator based on the Shimmy Frontend Contract. It provides a complete, functional React + Vite frontend that interfaces with your local Shimmy backend.

### Generated Features

- **🔍 Auto-Discovery**: Automatically finds and connects to your Shimmy backend
- **🔌 WebSocket Integration**: Full bi-directional communication with reconnection logic
- **📱 Responsive Design**: Works on desktop, tablet, and mobile devices
- **🎨 Dark Theme**: Modern dark UI with blue accents
- **⚡ Real-time Streaming**: Live streaming chat responses
- **📊 System Metrics**: Real-time CPU, memory, and GPU usage monitoring

### Schema-Generated Components


This theme includes the following default components:

- **ModelChooser**: Select and manage AI models
- **Chat**: Real-time chat with streaming responses  
- **Metrics**: System performance monitoring


### Schema Information

- **Contract Version**: 
- **Discovery Endpoint**: /api/discovery:${discovery.port}
- **WebSocket Endpoint**: /ws/console
- **Generated**: 2025-11-17T22:19:39.023Z

## Quick Start

1. **Install Dependencies**
   ```bash
   npm install
   ```

2. **Start Development Server**
   ```bash
   npm run dev
   ```

3. **Open in Browser**
   ```
   http://localhost:5173
   ```

## Prerequisites

- **Node.js**: Version 18 or higher
- **Shimmy Backend**: A running Shimmy instance on your local machine
- **Modern Browser**: Chrome, Firefox, Safari, or Edge

## Development

### Available Scripts

- `npm run dev` - Start development server with hot reload
- `npm run build` - Build for production
- `npm run preview` - Preview production build locally
- `npm run lint` - Run ESLint
- `npm run type-check` - Run TypeScript type checking

### Project Structure

```
test-theme-2/
├── src/
│   ├── components/          # UI components
│   │   ├── ModelChooser.tsx # Model selection interface
│   │   ├── Chat.tsx         # Chat interface with streaming
│   │   └── Metrics.tsx      # System metrics display
│   ├── hooks/               # Custom React hooks
│   │   ├── useDiscovery.ts  # Backend discovery logic
│   │   └── useWebSocket.ts  # WebSocket connection management
│   ├── App.tsx              # Main application component
│   ├── main.tsx             # Application entry point
│   └── index.css            # Global styles
├── public/                  # Static assets
├── package.json             # Dependencies and scripts
├── vite.config.ts          # Vite configuration
├── tsconfig.json           # TypeScript configuration
└── README.md               # This file
```

## Customization

### Styling

This theme uses **Tailwind CSS** for styling. You can customize the appearance by:

1. **Modifying Colors**: Edit the Tailwind color classes in components
2. **Adding Custom Styles**: Add styles to `src/index.css`
3. **Tailwind Config**: Customize `tailwind.config.js` for theme-wide changes

### Components

Each component is self-contained and can be customized:

- **ModelChooser**: Modify model display format and selection UI
- **Chat**: Customize message formatting and streaming animation
- **Metrics**: Add new metrics or change visualization

### Schema Updates

If your Shimmy backend schema changes:

1. **Regenerate Theme**: Run the theme generator again to get updated types
2. **Manual Updates**: Modify components to handle new message types
3. **Type Safety**: Update TypeScript interfaces if needed

## WebSocket Message Types


This theme supports the following default WebSocket messages:

- `get_models` - Request available models
- `models_response` - List of available models
- `select_model` - Select a model for use
- `model_selected` - Model selection confirmation
- `chat_request` - Send chat message
- `chat_token` - Streaming chat response token
- `generation_complete` - Chat generation finished
- `get_metrics` - Request system metrics
- `metrics_response` - Current system metrics


## Troubleshooting

### Connection Issues

1. **Backend Not Found**
   - Ensure Shimmy is running locally
   - Check if discovery port 11430 is accessible
   - Try restarting your Shimmy instance

2. **WebSocket Errors**
   - Verify WebSocket endpoint is correct
   - Check browser console for detailed error messages
   - Ensure no firewall blocking local connections

3. **Model Loading Issues**
   - Verify models are properly configured in Shimmy
   - Check Shimmy logs for model loading errors
   - Ensure sufficient system resources

### Development Issues

1. **TypeScript Errors**
   - Run `npm run type-check` to see detailed errors
   - Ensure all dependencies are installed
   - Check for schema/interface mismatches

2. **Build Failures**
   - Clear `node_modules` and reinstall: `rm -rf node_modules && npm install`
   - Check for conflicting dependencies
   - Ensure Node.js version compatibility

## Support

This theme was generated by the Shimmy Theme Generator. For issues:

1. **Schema Mismatches**: Regenerate the theme with updated schema
2. **Backend Issues**: Check your Shimmy backend configuration
3. **Theme Bugs**: Report issues to the Shimmy project

## License

MIT License - feel free to modify and distribute.

---

**Generated by Shimmy Theme Generator v1.0.0**  
**Author**: Claude Code  
**Generated**: 11/17/2025