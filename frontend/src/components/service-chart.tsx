import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, AreaChart, Area, TooltipProps } from 'recharts';
import { ServiceMetrics } from "@/lib/api";
import { TrendingUp, Activity } from "lucide-react";

interface ServiceChartProps {
  metrics: ServiceMetrics;
  className?: string;
}

interface CustomTooltipProps extends TooltipProps<number, string> {
  active?: boolean;
  payload?: Array<{
    value: number;
    name: string;
    color: string;
  }>;
  label?: string;
}

export function ServiceChart({ metrics, className }: ServiceChartProps) {
  const formatDate = (dateString: string) => {
    const date = new Date(dateString);
    return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
  };

  const formatUptime = (value: number) => `${value.toFixed(2)}%`;
  const formatLatency = (value: number) => `${value}ms`;

  const CustomTooltip = ({ active, payload, label }: CustomTooltipProps) => {
    if (active && payload && payload.length) {
      return (
        <div className="bg-background border rounded-lg shadow-lg p-3">
          <p className="font-medium">{label ? formatDate(label) : ''}</p>
          {payload.map((entry, index) => (
            <p key={index} className="text-sm" style={{ color: entry.color }}>
              {entry.name}: {entry.name === 'Uptime' ? formatUptime(entry.value) : formatLatency(entry.value)}
            </p>
          ))}
        </div>
      );
    }
    return null;
  };

  return (
    <Card className={className}>
      <CardHeader>
        <CardTitle className="text-lg">Performance Metrics</CardTitle>
      </CardHeader>
      <CardContent>
        <Tabs defaultValue="uptime" className="w-full">
          <TabsList className="grid w-full grid-cols-2">
            <TabsTrigger value="uptime" className="flex items-center gap-2">
              <TrendingUp className="w-4 h-4" />
              Uptime
            </TabsTrigger>
            <TabsTrigger value="latency" className="flex items-center gap-2">
              <Activity className="w-4 h-4" />
              Latency
            </TabsTrigger>
          </TabsList>
          
          <TabsContent value="uptime" className="mt-4">
            <div className="h-64">
              <ResponsiveContainer width="100%" height="100%">
                <AreaChart data={metrics.uptimeData}>
                  <CartesianGrid strokeDasharray="3 3" className="opacity-30" />
                  <XAxis 
                    dataKey="date" 
                    tickFormatter={formatDate}
                    className="text-xs"
                  />
                  <YAxis 
                    domain={[95, 100]}
                    tickFormatter={formatUptime}
                    className="text-xs"
                  />
                  <Tooltip content={<CustomTooltip />} />
                  <Area
                    type="monotone"
                    dataKey="uptime"
                    stroke="#10b981"
                    fill="#10b981"
                    fillOpacity={0.3}
                    strokeWidth={2}
                    name="Uptime"
                  />
                </AreaChart>
              </ResponsiveContainer>
            </div>
            <div className="mt-4 grid grid-cols-2 gap-4 text-center">
              <div>
                <p className="text-sm text-muted-foreground">Current Uptime</p>
                <p className="text-2xl font-bold text-green-600">{metrics.currentUptime.toFixed(2)}%</p>
              </div>
              <div>
                <p className="text-sm text-muted-foreground">30-Day Average</p>
                <p className="text-2xl font-bold">
                  {(metrics.uptimeData.reduce((sum, data) => sum + data.uptime, 0) / metrics.uptimeData.length).toFixed(2)}%
                </p>
              </div>
            </div>
          </TabsContent>
          
          <TabsContent value="latency" className="mt-4">
            <div className="h-64">
              <ResponsiveContainer width="100%" height="100%">
                <LineChart data={metrics.uptimeData}>
                  <CartesianGrid strokeDasharray="3 3" className="opacity-30" />
                  <XAxis 
                    dataKey="date" 
                    tickFormatter={formatDate}
                    className="text-xs"
                  />
                  <YAxis 
                    tickFormatter={formatLatency}
                    className="text-xs"
                  />
                  <Tooltip content={<CustomTooltip />} />
                  <Line
                    type="monotone"
                    dataKey="latency"
                    stroke="#3b82f6"
                    strokeWidth={2}
                    dot={{ fill: '#3b82f6', strokeWidth: 2, r: 3 }}
                    activeDot={{ r: 5, stroke: '#3b82f6', strokeWidth: 2 }}
                    name="Latency"
                  />
                </LineChart>
              </ResponsiveContainer>
            </div>
            <div className="mt-4 grid grid-cols-2 gap-4 text-center">
              <div>
                <p className="text-sm text-muted-foreground">Current Latency</p>
                <p className="text-2xl font-bold text-blue-600">{metrics.currentLatency}ms</p>
              </div>
              <div>
                <p className="text-sm text-muted-foreground">Average Latency</p>
                <p className="text-2xl font-bold">{metrics.averageLatency}ms</p>
              </div>
            </div>
          </TabsContent>
        </Tabs>
      </CardContent>
    </Card>
  );
} 