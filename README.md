# Client Application

A Rust client application that communicates with the server for database operations.

## Features

- Connects to the Rust server backend
- Handles user input and displays responses
- Implements client-side validation and error handling
- Supports secure communication with the server

## Getting Started

### Prerequisites

- Rust (latest stable version)
- Cargo package manager

### Installation

1. Navigate to the client directory:
   ```bash
   cd client
   ```

2. Build the project:
   ```bash
   cargo build
   ```

3. Run the application:
   ```bash
   cargo run
   ```

## Configuration

The client connects to the server using the following configuration:

- **Server URL**: Configure in environment variables or config file
- **Connection timeout**: Configurable timeout settings
- **Retry logic**: Built-in retry mechanism for failed requests

## Environment Variables

Create a `.env` file in the client directory with:

```env
SERVER_URL=http://localhost:8080
CLIENT_TIMEOUT=30
MAX_RETRIES=3
```

## Development

### Building for Production

```bash
cargo build --release
```

### Running Tests

```bash
cargo test
```

## Railway Deployment

This client will be deployed as a separate Railway service:

1. Create a new Railway project
2. Connect to your GitHub repository
3. Set environment variables in Railway dashboard
4. Deploy using Railway's automatic deployment

## API Communication

The client communicates with the server through REST API endpoints for:
- Data retrieval
- Data manipulation
- User authentication
- Real-time updates

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Submit a pull request 