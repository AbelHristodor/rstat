# 🚀 Rstat - A Rust Healthcheck Status Page

[![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Next.js](https://img.shields.io/badge/Next.js-000000?style=for-the-badge&logo=next.js&logoColor=white)](https://nextjs.org/)
[![PostgreSQL](https://img.shields.io/badge/PostgreSQL-316192?style=for-the-badge&logo=postgresql&logoColor=white)](https://www.postgresql.org/)
[![TypeScript](https://img.shields.io/badge/TypeScript-007ACC?style=for-the-badge&logo=typescript&logoColor=white)](https://www.typescriptlang.org/)
[![Tailwind CSS](https://img.shields.io/badge/Tailwind_CSS-38B2AC?style=for-the-badge&logo=tailwind-css&logoColor=white)](https://tailwindcss.com/)

[![License: GNU GPLv3](https://img.shields.io/badge/license-%20%20GNU%20GPLv3%20-green?style=plastic)](https://www.gnu.org/licenses/gpl-3.0.html)
[![Made with Love](https://img.shields.io/badge/Made%20with-❤️-red.svg?style=for-the-badge)](https://github.com/abelhristodor/rstat)

> **Real-time system monitoring with blazing fast performance** ⚡

Rstat is a modern, high-performance healthcheck monitoring system built with **Rust** for the backend and **Next.js** for the frontend. It provides real-time monitoring of your services with beautiful dashboards, detailed metrics, and instant notifications (soon).

<p align="center">
  <img src=".github/assets/img1-1.png" alt="Status Dashboard - Dark Mode" width="800"/>
</p>

<p align="center">
  <img src=".github/assets/img2-1.png" alt="Status Dashboard - Dark Mode" width="800"/>
</p>
<p align="center">
  <img src=".github/assets/img3-1.png" alt="Status Dashboard - Light Mode" width="800"/>
  <small style="opacity:1; font-size:90%;">Image captured using <a href="https://www.cleansnap.co/">CleanSnap</a>.</small>
</p>

## 🌟 Features

### 🔧 Backend (Rust)
- **⚡ Blazing Fast**: Built with Rust for maximum performance and memory safety
- **🔄 Async Health Checks**: HTTP and TCP health monitoring with configurable intervals
- **📊 PostgreSQL Storage**: Robust data persistence with SQLx
- **🛡️ Retry Logic**: Intelligent retry mechanisms with exponential backoff
- **📈 Real-time Metrics**: Response time tracking and uptime calculations
- **🔔 Notification System**: Built-in notification framework for alerts
- **🏗️ Modular Architecture**: Clean separation of concerns with async/await

### 🎨 Frontend (Next.js)
- **🎯 Modern UI**: Beautiful, responsive dashboard built with Tailwind CSS
- **📱 Mobile First**: Optimized for all device sizes
- **🌙 Dark Mode**: Automatic theme switching
- **📊 Interactive Charts**: Real-time performance visualization with Recharts
- **⚡ Fast Loading**: Optimized with Next.js 15 and Turbopack
- **🔍 Real-time Updates**: Live status updates without page refresh

## 🏗️ Architecture

The application is structured as a Rust workspace with the following crates:

### Core Crates

- **`rstat-core`** - Core domain models and shared types
  - Service and HealthCheck models
  - Common traits and enums
  - Shared data structures

- **`rstat-service`** - Service management
  - Database operations for services
  - Service CRUD operations
  - Service fixtures for testing

- **`rstat-healthcheck`** - Health checking functionality
  - HTTP and TCP health checkers
  - Health check result storage
  - Checker implementations

- **`rstat-metrics`** - Metrics calculation and storage
  - Service metrics models
  - Metrics calculation logic
  - Database operations for metrics

- **`rstat-scheduler`** - Scheduling and background tasks
  - Service scheduling logic
  - Metrics updater
  - Background task management

### Application Crates

- **`rstat-api`** - HTTP API layer
  - REST API endpoints
  - Request/response handling
  - API types and validation

- **`rstat-cli`** - Command line interface
  - CLI command definitions
  - Command parsing and routing

- **`rstat-seeder`** - Data seeding functionality
  - Test data generation
  - Database seeding
  - Sample service creation

- **`rstat-server`** - Main server application
  - Application entry point
  - Component orchestration
  - Server startup logic

- **`rstat-config`** - YAML configuration loader for services

## Benefits of This Structure

1. **Separation of Concerns**: Each crate has a single responsibility
2. **Reusability**: Core functionality can be reused across different applications
3. **Testability**: Each crate can be tested independently
4. **Maintainability**: Changes to one area don't affect others
5. **Scalability**: Easy to add new features or modify existing ones
6. **Dependency Management**: Clear dependency boundaries between components

## 🚀 Quick Start

### Prerequisites
- **Rust** (latest stable)
- **Node.js** 18+ 
- **PostgreSQL** 17+
- **Docker** (optional)

### 1. Clone the Repository
```bash
git clone https://github.com/abelhristodor/rstat.git
cd rstat
```

### 2. Set Up Infrastructure
```bash
# Start PostgreSQL with Docker
make infrastructure-up

# Or manually set up PostgreSQL and update DATABASE_URL in .env
```

### 3. Backend Setup
```bash
# Install dependencies
cargo build

# Run database migrations
make migrate

# Seed with sample data (optional)
make seed

# Start the backend server
make run
```

### 4. Frontend Setup
```bash
cd frontend

# Install dependencies
npm install
# or
bun install

# Start development server
npm run dev
# or
bun dev
```

### 5. Access the Application
- **Frontend**: http://localhost:3001
- **Backend API**: http://localhost:3000

## 📚 API Reference

### Health Check Endpoints

#### Create HTTP Health Check
```http
POST /http
Content-Type: application/json

{
  "name": "API Gateway",
  "kind": "HTTP",
  "url": "https://api.example.com/health",
  "interval": 30
}
```

#### List All Services
```http
GET /http
```

#### Get Health Check Results
```http
GET /http/checks/{service_id}
```

#### Delete Service
```http
DELETE /http
Content-Type: application/json

{
  "id": "service-uuid"
}
```

## 🛠️ Development

### Backend Development
```bash
# Run with hot reload
cargo watch -x run

# Run tests
cargo test

# Check code quality
cargo clippy
```

### Frontend Development
```bash
cd frontend

# Development with Turbopack
npm run dev

# Build for production
npm run build

# Start production server
npm start

# Lint code
npm run lint
```

### Database Management
```bash
# Run migrations
make migrate

# Reset database
make infrastructure-down-volumes
make infrastructure-up
make migrate
make seed
```

## 🐳 Docker Deployment

### Using Docker Compose
```bash
# Build and start all services
docker-compose up -d

# View logs
docker-compose logs -f

# Stop services
docker-compose down
```

### Environment Variables
Create a `.env` file in the root directory:
```env
DATABASE_URL=postgresql://postgres:postgres@localhost:5432/postgres
RUST_LOG=rstat=info,tower_http=debug
```

## 📊 Monitoring Features

### Health Check Types
- **HTTP Checks**: Monitor web services and APIs
- **TCP Checks**: Monitor database connections and custom ports
- **Custom Headers**: Support for authentication and custom headers
- **Response Validation**: Check response codes and content

### Metrics & Analytics
- **Uptime Tracking**: Real-time uptime percentage calculation
- **Response Time**: Latency monitoring with historical data
- **Status History**: Complete audit trail of all health checks
- **Performance Charts**: Interactive visualizations of system performance

### Alerting & Notifications
- **Real-time Alerts**: Instant notification of service outages
- **Status Changes**: Notifications when services change state
- **Customizable Thresholds**: Configurable alert conditions

## 🎨 UI Components

The frontend uses a modern component library built with:
- **Tailwind CSS**: Utility-first CSS framework
- **Lucide React**: Beautiful, customizable icons
- **Recharts**: Responsive charting library
- **Next.js 15**: Latest React framework with App Router

## 🔧 Configuration

### Service Configuration
```rust
// Example service configuration
{
  "name": "My API",
  "kind": "HTTP",
  "url": "https://api.example.com/health",
  "interval": 30,
  "timeout": 5,
  "retries": 3,
  "headers": {
    "Authorization": "Bearer token"
  }
}
```

### Scheduler Configuration
The scheduler runs health checks based on configured intervals:
- **Automatic Scheduling**: Services are checked at their specified intervals
- **Concurrent Execution**: Multiple health checks run simultaneously
- **Error Handling**: Failed checks are retried with exponential backoff

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Setup
1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request


## ⚡ YAML Service Configuration

You can now define your services in a YAML file and have them created automatically on startup or via CLI.

### Example YAML (`config/services.yaml`):
```yaml
- name: "API Gateway"
  kind:
    type: http
    url: "https://api.example.com/health"
    method: "GET"
    timeout: 5
    max_retries: 3
  interval: 30

- name: "Database Cluster"
  kind:
    type: tcp
    host: "db.example.com"
    port: 5432
    timeout: 5
    max_retries: 3
  interval: 60
```

### How it works
- On startup, the backend will look for a YAML file in `config/services.yaml` or as specified by the `RSTAT_CONFIG_PATH` environment variable.
- All services defined in the file will be created in the database if they do not already exist.

### Manual Loading
You can also load services from YAML at any time:

```bash
# Load from a specific file
make config-load FILE=config/services.yaml

# Load from a directory of YAML files
make config-load-dir DIR=config

# Load from default locations (config/services.yaml, etc.)
make config-load-default
```

Or use the CLI directly:
```bash
cargo run -p rstat-server -- config load --file config/services.yaml
```