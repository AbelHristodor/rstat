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

// Helper function to convert ServiceMetricsSummary to ServiceMetrics
function convertMetricsSummaryToServiceMetrics(summary: ServiceMetricsSummary): ServiceMetrics {
  return {
    serviceId: summary.service_id,
    uptimeData: summary.uptime_data.map(point => ({
      date: point.date,
      uptime: point.uptime_percentage,
      latency: point.latency_ms
    })),
    currentUptime: summary.current_uptime,
    currentLatency: summary.current_latency_ms,
    averageLatency: summary.average_latency_ms
  };
}

// API functions
export async function fetchServices(): Promise<ServiceStatus[]> {
  try {
    const response = await fetchWithTimeout(`${API_BASE_URL}/http`);
    if (!response.ok) {
      if (response.status === 404) {
        throw new Error('API endpoint not found. Please check if the backend service is running.');
      } else if (response.status >= 500) {
        throw new Error(`Server error (${response.status}): The backend service is experiencing issues.`);
      } else {
        throw new Error(`HTTP error! status: ${response.status}`);
      }
    }
    const services: Service[] = await response.json();
    
    // For each service, fetch its metrics summary to get current status
    const servicesWithMetrics = await Promise.all(
      services.map(async (service) => {
        try {
          const metricsResponse = await fetchWithTimeout(`${API_BASE_URL}/metrics/${service.id}/summary`);
          if (metricsResponse.ok) {
            const metrics: ServiceMetricsSummary = await metricsResponse.json();
            return convertServiceToStatus(service, metrics);
          }
        } catch (error) {
          console.warn(`Failed to fetch metrics for service ${service.id}:`, error);
        }
        return convertServiceToStatus(service);
      })
    );
    
    return servicesWithMetrics;
  } catch (error) {
    console.error('Error fetching services:', error);
    if (error instanceof Error) {
      if (error.message.includes('Failed to fetch') || error.message.includes('NetworkError')) {
        throw new Error('Unable to connect to the backend service. Please check your network connection and ensure the service is running.');
      }
      throw error;
    }
    throw new Error('An unexpected error occurred while fetching services.');
  }
}

export async function fetchServiceMetrics(serviceId: string, days: number = 30): Promise<ServiceMetrics> {
  try {
    const response = await fetchWithTimeout(`${API_BASE_URL}/metrics/${serviceId}/summary?days=${days}`);
    if (!response.ok) {
      if (response.status === 404) {
        throw new Error(`Metrics not found for service ${serviceId}. The service may not exist or have no metrics data.`);
      } else if (response.status >= 500) {
        throw new Error(`Server error (${response.status}): Unable to fetch metrics for service ${serviceId}.`);
      } else {
        throw new Error(`HTTP error! status: ${response.status}`);
      }
    }
    const summary: ServiceMetricsSummary = await response.json();
    return convertMetricsSummaryToServiceMetrics(summary);
  } catch (error) {
    console.error(`Error fetching metrics for service ${serviceId}:`, error);
    if (error instanceof Error) {
      if (error.message.includes('Failed to fetch') || error.message.includes('NetworkError')) {
        throw new Error('Unable to connect to the backend service. Please check your network connection.');
      }
      throw error;
    }
    throw new Error(`An unexpected error occurred while fetching metrics for service ${serviceId}.`);
  }
}

export async function fetchAllServiceMetrics(days: number = 30): Promise<ServiceMetrics[]> {
  try {
    const services = await fetchServices();
    const metrics = await Promise.all(
      services.map(async (service) => {
        try {
          return await fetchServiceMetrics(service.id, days);
        } catch (error) {
          console.warn(`Failed to fetch metrics for service ${service.id}:`, error);
          // Return empty metrics if fetch fails
          return {
            serviceId: service.id,
            uptimeData: [],
            currentUptime: 0,
            currentLatency: 0,
            averageLatency: 0
          };
        }
      })
    );
    return metrics;
  } catch (error) {
    console.error('Error fetching all service metrics:', error);
    throw error;
  }
}

// Additional API functions for direct backend access
export async function fetchRawServices(): Promise<Service[]> {
  try {
    const response = await fetchWithTimeout(`${API_BASE_URL}/http`);
    if (!response.ok) {
      if (response.status === 404) {
        throw new Error('API endpoint not found. Please check if the backend service is running.');
      } else if (response.status >= 500) {
        throw new Error(`Server error (${response.status}): The backend service is experiencing issues.`);
      } else {
        throw new Error(`HTTP error! status: ${response.status}`);
      }
    }
    return response.json();
  } catch (error) {
    console.error('Error fetching raw services:', error);
    if (error instanceof Error) {
      if (error.message.includes('Failed to fetch') || error.message.includes('NetworkError')) {
        throw new Error('Unable to connect to the backend service. Please check your network connection.');
      }
      throw error;
    }
    throw new Error('An unexpected error occurred while fetching services.');
  }
}

export async function fetchRawServiceMetrics(serviceId: string, days: number = 30): Promise<ServiceMetric[]> {
  try {
    const response = await fetchWithTimeout(`${API_BASE_URL}/metrics/${serviceId}?days=${days}`);
    if (!response.ok) {
      if (response.status === 404) {
        throw new Error(`Metrics not found for service ${serviceId}. The service may not exist or have no metrics data.`);
      } else if (response.status >= 500) {
        throw new Error(`Server error (${response.status}): Unable to fetch metrics for service ${serviceId}.`);
      } else {
        throw new Error(`HTTP error! status: ${response.status}`);
      }
    }
    return response.json();
  } catch (error) {
    console.error(`Error fetching raw metrics for service ${serviceId}:`, error);
    if (error instanceof Error) {
      if (error.message.includes('Failed to fetch') || error.message.includes('NetworkError')) {
        throw new Error('Unable to connect to the backend service. Please check your network connection.');
      }
      throw error;
    }
    throw new Error(`An unexpected error occurred while fetching metrics for service ${serviceId}.`);
  }
}

export async function fetchRawServiceMetricsSummary(serviceId: string, days: number = 30): Promise<ServiceMetricsSummary> {
  try {
    const response = await fetchWithTimeout(`${API_BASE_URL}/metrics/${serviceId}/summary?days=${days}`);
    if (!response.ok) {
      if (response.status === 404) {
        throw new Error(`Metrics summary not found for service ${serviceId}. The service may not exist or have no metrics data.`);
      } else if (response.status >= 500) {
        throw new Error(`Server error (${response.status}): Unable to fetch metrics summary for service ${serviceId}.`);
      } else {
        throw new Error(`HTTP error! status: ${response.status}`);
      }
    }
    return response.json();
  } catch (error) {
    console.error(`Error fetching raw metrics summary for service ${serviceId}:`, error);
    if (error instanceof Error) {
      if (error.message.includes('Failed to fetch') || error.message.includes('NetworkError')) {
        throw new Error('Unable to connect to the backend service. Please check your network connection.');
      }
      throw error;
    }
    throw new Error(`An unexpected error occurred while fetching metrics summary for service ${serviceId}.`);
  }
}