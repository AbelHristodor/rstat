// API base URL - should be configurable via environment variables
const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3001';

// Timeout configuration (10 seconds)
const API_TIMEOUT = 10000;

// Helper function to create a fetch request with timeout
async function fetchWithTimeout(url: string, options: RequestInit = {}): Promise<Response> {
  const controller = new AbortController();
  const timeoutId = setTimeout(() => controller.abort(), API_TIMEOUT);
  
  try {
    const response = await fetch(url, {
      ...options,
      signal: controller.signal,
    });
    clearTimeout(timeoutId);
    return response;
  } catch (error) {
    clearTimeout(timeoutId);
    if (error instanceof Error && error.name === 'AbortError') {
      throw new Error(`Request timeout: The server took too long to respond (${API_TIMEOUT / 1000}s)`);
    }
    throw error;
  }
}

// Types that match the Rust backend
export interface Service {
  id: string;
  name: string;
  kind: {
    HTTP?: any;
    TCP?: any;
  };
  interval: {
    secs: number;
    nanos: number;
  };
  next_run: string; // ISO datetime string
}

export interface ServiceMetric {
  id: string;
  service_id: string;
  date: string; // YYYY-MM-DD format
  uptime_percentage: number;
  average_latency_ms: number;
  total_checks: number;
  successful_checks: number;
  created_at: string; // ISO datetime string
  updated_at: string; // ISO datetime string
}

export interface ServiceMetricsSummary {
  service_id: string;
  current_uptime: number;
  current_latency_ms: number;
  average_latency_ms: number;
  uptime_data: UptimeDataPoint[];
}

export interface UptimeDataPoint {
  date: string; // YYYY-MM-DD format
  uptime_percentage: number;
  latency_ms: number;
}

// Frontend-specific types for compatibility
export interface ServiceStatus {
  id: string;
  name: string;
  status: 'operational' | 'degraded' | 'outage' | 'maintenance';
  uptime: number; // percentage
  latency: number; // in milliseconds
  lastUpdated: string;
  description: string;
}

export interface UptimeData {
  date: string;
  uptime: number;
  latency: number;
}

export interface ServiceMetrics {
  serviceId: string;
  uptimeData: UptimeData[];
  currentUptime: number;
  currentLatency: number;
  averageLatency: number;
}

// Helper function to determine service status based on uptime
function getServiceStatus(uptime: number): 'operational' | 'degraded' | 'outage' | 'maintenance' {
  if (uptime >= 99.9) return 'operational';
  if (uptime >= 95.0) return 'degraded';
  if (uptime >= 90.0) return 'maintenance';
  return 'outage';
}

// Helper function to convert Service to ServiceStatus
function convertServiceToStatus(service: Service, metrics?: ServiceMetricsSummary): ServiceStatus {
  const currentUptime = metrics?.current_uptime || 0;
  const currentLatency = metrics?.current_latency_ms || 0;
  
  // Extract the kind type (HTTP or TCP)
  const kindType = service.kind.HTTP ? 'HTTP' : service.kind.TCP ? 'TCP' : 'Unknown';
  
  // Extract interval in seconds
  const intervalSeconds = service.interval.secs;
  
  return {
    id: service.id,
    name: service.name,
    status: getServiceStatus(currentUptime),
    uptime: currentUptime,
    latency: currentLatency,
    lastUpdated: service.next_run,
    description: `${kindType} service monitored every ${intervalSeconds} seconds`
  };
}

export interface ServiceWithMetricsSummary {
  service: Service;
  metrics_summary: ServiceMetricsSummary;
}

export async function fetchServicesWithMetrics(days: number = 30): Promise<{ status: ServiceStatus, metrics: ServiceMetrics }[]> {
  try {
    const response = await fetchWithTimeout(`${API_BASE_URL}/services_with_metrics?days=${days}`);
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }
    const data: ServiceWithMetricsSummary[] = await response.json();
    return data.map(({ service, metrics_summary }) => ({
      status: convertServiceToStatus(service, metrics_summary),
      metrics: {
        serviceId: service.id,
        uptimeData: metrics_summary.uptime_data.map(point => ({
          date: point.date,
          uptime: point.uptime_percentage,
          latency: point.latency_ms
        })),
        currentUptime: metrics_summary.current_uptime,
        currentLatency: metrics_summary.current_latency_ms,
        averageLatency: metrics_summary.average_latency_ms
      }
    }));
  } catch (error) {
    console.error('Error fetching services with metrics:', error);
    throw error;
  }
}