# IronVein Web Client

A modern web client built with Rust WebAssembly (WASM) and Vite that communicates with the IronVein server for database operations.

## 🚀 Features

- **Rust + WebAssembly**: Core logic written in Rust and compiled to WASM for high performance
- **Modern Web UI**: Clean, responsive interface built with HTML5 and CSS3
- **Real-time Communication**: Connects to the Rust server backend via REST API
- **Live Development**: Hot-reload development server with Vite
- **Railway Deployment**: Optimized for Railway cloud deployment

## 🛠️ Technology Stack

- **Rust**: Backend logic compiled to WebAssembly
- **Vite**: Build tool and development server
- **WebAssembly**: High-performance web execution
- **JavaScript**: Frontend integration and DOM manipulation
- **HTML/CSS**: Modern responsive UI

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

1. **Configure Server Connection**: Set the URL of your IronVein server
2. **Test Connectivity**: Verify connection to the backend
3. **Fetch Data**: Retrieve data from the server/database
4. **Interactive Features**: Real-time updates and user interactions

### Web Interface Features:

- **Server URL Configuration**: Dynamic server endpoint configuration
- **Connection Testing**: Built-in connectivity diagnostics  
- **Data Fetching**: Retrieve and display server data
- **Real-time Logging**: Live activity feed with timestamps
- **Responsive Design**: Works on desktop and mobile devices

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
├── index.html              # Main HTML file
├── main.js                 # JavaScript entry point
├── vite.config.js          # Vite configuration
├── package.json            # Node.js dependencies
├── Cargo.toml              # Rust dependencies
└── README.md               # This file
```

## 🔍 API Communication

The client communicates with the server through:

- **REST API endpoints**: HTTP requests to server endpoints
- **JSON data format**: Structured data exchange
- **Error handling**: Comprehensive error reporting
- **Connection management**: Automatic retry logic

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
3. Make your changes
4. Build and test: `npm run build`
5. Commit changes: `git commit -m "Add feature"`
6. Push to branch: `git push origin feature-name`
7. Submit a pull request

## 📊 Performance

- **Fast Loading**: WebAssembly provides near-native performance
- **Small Bundle Size**: Optimized WASM modules
- **Efficient Updates**: Vite's hot module replacement
- **Responsive UI**: Smooth user interactions

## 🔧 Troubleshooting

### Common Issues:

1. **WASM Module Failed to Load**
   - Ensure `npm run build:wasm` completed successfully
   - Check browser console for detailed error messages

2. **Server Connection Failed**
   - Verify server URL is correct
   - Check if server is running and accessible
   - Ensure CORS is properly configured on server

3. **Build Errors**
   - Update Rust toolchain: `rustup update`
   - Clear cache: `npm run build:wasm` then `npm run dev` 