# Frontend API Integration Setup

This frontend application has been updated to integrate with the Rust backend API instead of using mock data.

## Environment Configuration

Create a `.env.local` file in the frontend directory with the following content:

```env
# API Configuration
NEXT_PUBLIC_API_URL=http://localhost:3001
```

## API Endpoints

The frontend now uses the following Rust backend endpoints:

- `GET /http` - List all services
- `GET /metrics/{service_id}` - Get metrics for a specific service
- `GET /metrics/{service_id}/summary` - Get metrics summary for a specific service

## Running the Application

1. Start the Rust backend:
   ```bash
   cd /path/to/rstat
   cargo run
   ```

2. Start the frontend:
   ```bash
   cd frontend
   npm run dev
   ```

3. The frontend will automatically connect to the Rust backend at `http://localhost:3001`

## Data Flow

1. The frontend fetches services from `/http`
2. For each service, it fetches metrics summary from `/metrics/{service_id}/summary`
3. The data is converted to the frontend's expected format
4. Charts and status cards are updated with real data

## Error Handling

- If the backend is not available, the frontend will show error messages
- Individual service metrics failures are handled gracefully
- The application continues to work even if some services fail to load

## Development

To modify the API integration, edit `src/lib/api.ts`. The file includes:
- Type definitions matching the Rust backend
- Conversion functions between backend and frontend formats
- Error handling and fallback logic 