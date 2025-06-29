# 🚀 Rstat - A Rust Healthcheck Status Page

[![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Next.js](https://img.shields.io/badge/Next.js-000000?style=for-the-badge&logo=next.js&logoColor=white)](https://nextjs.org/)
[![PostgreSQL](https://img.shields.io/badge/PostgreSQL-316192?style=for-the-badge&logo=postgresql&logoColor=white)](https://www.postgresql.org/)
[![TypeScript](https://img.shields.io/badge/TypeScript-007ACC?style=for-the-badge&logo=typescript&logoColor=white)](https://www.typescriptlang.org/)
[![Tailwind CSS](https://img.shields.io/badge/Tailwind_CSS-38B2AC?style=for-the-badge&logo=tailwind-css&logoColor=white)](https://tailwindcss.com/)

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=for-the-badge)](https://opensource.org/licenses/MIT)
[![Made with Love](https://img.shields.io/badge/Made%20with-❤️-red.svg?style=for-the-badge)](https://github.com/abelhristodor/rstat)

> **Real-time system monitoring with blazing fast performance** ⚡

Rstat is a modern, high-performance healthcheck monitoring system built with **Rust** for the backend and **Next.js** for the frontend. It provides real-time monitoring of your services with beautiful dashboards, detailed metrics, and instant notifications.

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
- **🌙 Dark Mode**: Automatic theme switching with next-themes
- **📊 Interactive Charts**: Real-time performance visualization with Recharts
- **⚡ Fast Loading**: Optimized with Next.js 15 and Turbopack
- **🔍 Real-time Updates**: Live status updates without page refresh
- **🎨 Radix UI**: Accessible, customizable components

## 🏗️ Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Next.js       │    │   Rust Backend  │    │   PostgreSQL    │
│   Frontend      │◄──►│   (Axum)        │◄──►│   Database      │
│                 │    │                 │    │                 │
│ • Status Page   │    │ • Health Checks │    │ • Services      │
│ • Real-time UI  │    │ • API Endpoints │    │ • Results       │
│ • Charts        │    │ • Scheduler     │    │ • Metrics       │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

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
- **Radix UI**: Accessible, unstyled components
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

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.


---

**Made with ❤️ by [Abel Hristodor](https://github.com/abelhristodor)**

*Star this repository if you found it helpful! ⭐*