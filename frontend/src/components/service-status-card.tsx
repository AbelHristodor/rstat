import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Progress } from "@/components/ui/progress";
import { ServiceStatus } from "@/lib/api";
import { Activity, Clock, TrendingUp } from "lucide-react";

interface ServiceStatusCardProps {
  service: ServiceStatus;
  className?: string;
}

const statusConfig = {
  operational: {
    label: 'Operational',
    color: 'bg-green-500',
    textColor: 'text-green-500',
    bgColor: 'bg-green-50 dark:bg-green-950/20'
  },
  degraded: {
    label: 'Degraded',
    color: 'bg-yellow-500',
    textColor: 'text-yellow-500',
    bgColor: 'bg-yellow-50 dark:bg-yellow-950/20'
  },
  outage: {
    label: 'Outage',
    color: 'bg-red-500',
    textColor: 'text-red-500',
    bgColor: 'bg-red-50 dark:bg-red-950/20'
  },
  maintenance: {
    label: 'Maintenance',
    color: 'bg-blue-500',
    textColor: 'text-blue-500',
    bgColor: 'bg-blue-50 dark:bg-blue-950/20'
  }
};

export function ServiceStatusCard({ service, className }: ServiceStatusCardProps) {
  const config = statusConfig[service.status];
  
  return (
    <Card className={`transition-all duration-200 hover:shadow-lg ${className}`}>
      <CardHeader className="pb-3">
        <div className="flex items-center justify-between">
          <CardTitle className="text-lg font-semibold">{service.name}</CardTitle>
          <Badge 
            variant="secondary" 
            className={`${config.bgColor} ${config.textColor} border-0 font-medium`}
          >
            <div className={`w-2 h-2 rounded-full ${config.color} mr-2`} />
            {config.label}
          </Badge>
        </div>
        <p className="text-sm text-muted-foreground">{service.description}</p>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="grid grid-cols-2 gap-4">
          <div className="space-y-2">
            <div className="flex items-center gap-2 text-sm text-muted-foreground">
              <TrendingUp className="w-4 h-4" />
              Uptime
            </div>
            <div className="flex items-baseline gap-1">
              <span className="text-2xl font-bold">{service.uptime.toFixed(2)}%</span>
            </div>
            <Progress value={service.uptime} className="h-2" />
          </div>
          
          <div className="space-y-2">
            <div className="flex items-center gap-2 text-sm text-muted-foreground">
              <Activity className="w-4 h-4" />
              Latency
            </div>
            <div className="flex items-baseline gap-1">
              <span className="text-2xl font-bold">{service.latency}</span>
              <span className="text-sm text-muted-foreground">ms</span>
            </div>
            <div className="h-2 bg-muted rounded-full">
              <div 
                className={`h-full rounded-full transition-all duration-300 ${
                  service.latency < 50 ? 'bg-green-500' :
                  service.latency < 100 ? 'bg-yellow-500' : 'bg-red-500'
                }`}
                style={{ width: `${Math.min(100, (service.latency / 200) * 100)}%` }}
              />
            </div>
          </div>
        </div>
        
        <div className="flex items-center gap-2 text-xs text-muted-foreground">
          <Clock className="w-3 h-3" />
          Last updated: {new Date(service.lastUpdated).toLocaleString()}
        </div>
      </CardContent>
    </Card>
  );
} 