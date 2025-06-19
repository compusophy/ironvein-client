# IronVein Web Client

A modern web client built with Rust WebAssembly (WASM) and Vite that communicates with the IronVein server for database operations.

## 🎨 IronVein Style Guide

### Theme Colors
```css
:root {
    --iron-primary: #2c3e50;    /* Main dark blue-gray */
    --iron-secondary: #34495e;  /* Secondary dark */
    --iron-accent: #e74c3c;     /* Iron red/crimson */
    --iron-success: #27ae60;    /* Success green */
    --iron-warning: #f39c12;    /* Warning orange */
    --iron-info: #3498db;       /* Info blue */
    --iron-light: #ecf0f1;      /* Light text */
    --iron-dark: #1a252f;       /* Dark backgrounds */
    --iron-border: #5d6d7e;     /* Border color */
    --iron-shadow: rgba(0, 0, 0, 0.3);  /* Shadows */
    --iron-glow: rgba(231, 76, 60, 0.3); /* Red glow effect */
}
```

### Design Principles
- **Industrial Theme**: Dark colors with red accents reflecting iron/steel
- **High Contrast**: Excellent readability for gaming sessions
- **Performance First**: Vanilla CSS for maximum speed
- **Mobile Responsive**: Grid-based layout that adapts to all devices
- **Accessibility**: Clear visual hierarchy and color coding

### Typography
- **Font Family**: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif
- **Headings**: Bold with iron-accent color (#e74c3c)
- **Body Text**: iron-light color (#ecf0f1)
- **Code/Monospace**: 'Courier New' for coordinates and technical data

### UI Components
- **Buttons**: Gradient backgrounds with hover animations
- **Inputs**: Dark backgrounds with glowing red focus states
- **Panels**: iron-secondary backgrounds with subtle borders
- **Health Bars**: Gradient from green to orange
- **Chat**: Dark scrollable areas with red accent scrollbars

### Layout Grid
- **Desktop**: 3-column layout (HP | Game | Players/Chat)
- **Mobile**: Single column stack (responsive grid areas)
- **Spacing**: 10px gaps between major components
- **Padding**: 15px internal padding for panels

### Animation & Effects
- **Hover States**: Subtle transforms (translateY(-1px to -2px))
- **Focus States**: Glowing box-shadows using iron-glow
- **Transitions**: 0.2-0.3s ease for smooth interactions
- **Pulse Animation**: For status indicators (2s infinite)

## 🚀 Features

- **Rust + WebAssembly**: Core logic written in Rust and compiled to WASM for high performance
- **Modern Web UI**: Clean, responsive interface built with HTML5 and CSS3
- **Real-time Communication**: Connects to the Rust server backend via WebSocket
- **Live Development**: Hot-reload development server with Vite
- **Railway Deployment**: Optimized for Railway cloud deployment
- **Mobile Responsive**: Grid-based layout that works on all devices
- **Online Players List**: Real-time display of connected players
- **Military Time Chat**: Timestamps in HH:MM:SS.MS format
- **Automatic Ping Display**: Real-time latency monitoring

## 🛠️ Technology Stack

- **Rust**: Backend logic compiled to WebAssembly
- **Vite**: Build tool and development server
- **WebAssembly**: High-performance web execution
- **JavaScript**: Frontend integration and DOM manipulation
- **HTML/CSS**: Modern responsive UI following IronVein theme

## 📋 Prerequisites

- Rust (latest stable version)
- Node.js (v16 or higher)
- npm or yarn package manager
- wasm-pack (for building WASM modules)

## 🔧 Installation & Setup

1. **Navigate to the client directory:**
   ```bash
   cd client
   ```

2. **Install Node.js dependencies:**
   ```bash
   npm install
   ```

3. **Build the WASM module:**
   ```bash
   npm run build:wasm
   ```

4. **Start the development server:**
   ```bash
   npm run dev
   ```

   The application will be available at `http://localhost:3000`

## 🏗️ Available Scripts

- `npm run dev` - Start development server with hot reload
- `npm run build` - Build for production deployment
- `npm run build:wasm` - Build only the WASM module
- `npm run preview` - Preview production build locally

## 🌐 Usage

The web client provides an intuitive interface to:

1. **Join Battles**: Enter your warrior name and select a battlefield
2. **Real-time Movement**: Click on the grid to move your unit
3. **Battle Chat**: Communicate with other players in real-time
4. **Combat Stats**: Monitor your health and resources with visual bars
5. **Online Players**: See who's currently online and their positions
6. **Ping Monitoring**: Real-time latency display with color coding

### Web Interface Features:

- **Responsive Design**: Works perfectly on desktop and mobile
- **Military Time**: Chat timestamps in precise HH:MM:SS.MS format
- **Auto Ping**: Continuous latency monitoring every 2 seconds
- **Player Tracking**: Live list of online players with positions
- **Visual Health Bars**: Gradient health and resource indicators
- **Dark Theme**: Easy on the eyes for long gaming sessions

## ⚙️ Configuration

### Environment Variables

The client automatically detects the server URL, but you can configure it via:

- Web interface server URL input field
- Environment variables (if needed for build-time configuration)

### Server Connection

Default server URL: `http://localhost:8080`

For production, update to your Railway server URL:
```
https://ironvein-server-production.up.railway.app
```

## 🚀 Railway Deployment

This client is optimized for Railway deployment:

### Build Configuration:
- **Port**: 3000 (configurable via PORT environment variable)
- **Static Files**: Served from `dist/` directory
- **Build Command**: `npm run build`
- **Start Command**: `npx vite preview --port $PORT --host 0.0.0.0`

### Deployment Steps:

1. **Push to GitHub**: Commit your changes to the ironvein-client repository
2. **Connect to Railway**: Link your GitHub repository to Railway
3. **Set Environment Variables**: Configure any needed environment variables
4. **Deploy**: Railway will automatically build and deploy your application

### Environment Variables for Railway:
```env
NODE_ENV=production
PORT=3000
VITE_SERVER_URL=https://your-server-url.railway.app
```

## 📁 Project Structure

```
client/
├── src/
│   ├── lib.rs              # Rust WASM library
│   └── main.rs             # Legacy file (not used in WASM)
├── pkg/                    # Generated WASM files (auto-generated)
├── dist/                   # Production build output
├── index.html              # Main HTML file with IronVein theme
├── main.js                 # JavaScript entry point
├── vite.config.js          # Vite configuration
├── package.json            # Node.js dependencies
├── Cargo.toml              # Rust dependencies
└── README.md               # This file
```

## 🔍 API Communication

The client communicates with the server through:

- **WebSocket connections**: Real-time bidirectional communication
- **JSON message protocol**: Structured data exchange
- **Automatic reconnection**: Robust connection management
- **Message queuing**: Handles network interruptions gracefully

## 🛠️ Development

### Local Development:
```bash
# Start development server
npm run dev

# Build for testing
npm run build

# Preview production build
npm run preview
```

### Adding New Features:
1. Add Rust functions to `src/lib.rs`
2. Rebuild WASM: `npm run build:wasm`
3. Update JavaScript in `main.js`
4. Test in browser at `http://localhost:3000`

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature-name`
3. Follow the IronVein style guide for any UI changes
4. Make your changes
5. Build and test: `npm run build`
6. Commit changes: `git commit -m "Add feature"`
7. Push to branch: `git push origin feature-name`
8. Submit a pull request

## 📊 Performance

- **Fast Loading**: WebAssembly provides near-native performance
- **Small Bundle Size**: Optimized WASM modules (~93KB gzipped)
- **Efficient Updates**: Vite's hot module replacement
- **Responsive UI**: Smooth 60fps interactions
- **Low Latency**: Direct WebSocket communication
- **Scalable**: Designed for 1000+ concurrent players

## 🔧 Troubleshooting

### Common Issues:

1. **WASM Module Failed to Load**
   - Ensure `npm run build:wasm` completed successfully
   - Check browser console for detailed error messages

2. **Server Connection Failed**
   - Verify server URL is correct
   - Check if server is running and accessible
   - Ensure CORS is properly configured on server

3. **Click to Move Not Working**
   - Ensure WebSocket connection is established
   - Check browser console for JavaScript errors
   - Verify canvas click handlers are properly registered

4. **Build Errors**
   - Update Rust toolchain: `rustup update`
   - Clear cache: `npm run build:wasm` then `npm run dev` 