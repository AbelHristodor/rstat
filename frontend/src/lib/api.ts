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

// Mock data for services
const mockServices: ServiceStatus[] = [
  {
    id: 'api-gateway',
    name: 'API Gateway',
    status: 'operational',
    uptime: 99.98,
    latency: 45,
    lastUpdated: new Date().toISOString(),
    description: 'Main API gateway service'
  },
  {
    id: 'database',
    name: 'Database',
    status: 'operational',
    uptime: 99.95,
    latency: 12,
    lastUpdated: new Date().toISOString(),
    description: 'Primary database cluster'
  },
  {
    id: 'auth-service',
    name: 'Authentication Service',
    status: 'operational',
    uptime: 99.99,
    latency: 28,
    lastUpdated: new Date().toISOString(),
    description: 'User authentication and authorization'
  },
  {
    id: 'file-storage',
    name: 'File Storage',
    status: 'degraded',
    uptime: 98.5,
    latency: 150,
    lastUpdated: new Date().toISOString(),
    description: 'File upload and storage service'
  },
  {
    id: 'email-service',
    name: 'Email Service',
    status: 'operational',
    uptime: 99.8,
    latency: 85,
    lastUpdated: new Date().toISOString(),
    description: 'Email delivery and notifications'
  },
  {
    id: 'cdn',
    name: 'CDN',
    status: 'operational',
    uptime: 99.99,
    latency: 8,
    lastUpdated: new Date().toISOString(),
    description: 'Content delivery network'
  }
];

// Generate mock uptime data for the last 30 days
function generateMockUptimeData(days: number = 30): UptimeData[] {
  const data: UptimeData[] = [];
  const baseUptime = 99.5;
  const baseLatency = 50;
  
  for (let i = days - 1; i >= 0; i--) {
    const date = new Date();
    date.setDate(date.getDate() - i);
    
    // Add some realistic variation
    const uptimeVariation = (Math.random() - 0.5) * 0.5; // ±0.25%
    const latencyVariation = (Math.random() - 0.5) * 20; // ±10ms
    
    data.push({
      date: date.toISOString().split('T')[0],
      uptime: Math.max(95, Math.min(100, baseUptime + uptimeVariation)),
      latency: Math.max(5, baseLatency + latencyVariation)
    });
  }
  
  return data;
}

// Mock API functions
export async function fetchServices(): Promise<ServiceStatus[]> {
  // Simulate API delay
  await new Promise(resolve => setTimeout(resolve, 300));
  return mockServices;
}

export async function fetchServiceMetrics(serviceId: string, days: number = 30): Promise<ServiceMetrics> {
  // Simulate API delay
  await new Promise(resolve => setTimeout(resolve, 200));
  
  const service = mockServices.find(s => s.id === serviceId);
  if (!service) {
    throw new Error(`Service ${serviceId} not found`);
  }
  
  const uptimeData = generateMockUptimeData(days);
  const averageLatency = uptimeData.reduce((sum, data) => sum + data.latency, 0) / uptimeData.length;
  
  return {
    serviceId,
    uptimeData,
    currentUptime: service.uptime,
    currentLatency: service.latency,
    averageLatency: Math.round(averageLatency)
  };
}

export async function fetchAllServiceMetrics(days: number = 30): Promise<ServiceMetrics[]> {
  const services = await fetchServices();
  const metrics = await Promise.all(
    services.map(service => fetchServiceMetrics(service.id, days))
  );
  return metrics;
} 