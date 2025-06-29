import { AlertTriangle, RefreshCw, Wifi, WifiOff } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { ThemeToggle } from "@/components/theme-toggle";

interface ErrorPageProps {
  error?: string;
  onRetry: () => void;
  isLoading?: boolean;
}

export function ErrorPage({ error, onRetry, isLoading = false }: ErrorPageProps) {
  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-50 to-slate-100 dark:from-slate-900 dark:to-slate-800 flex items-center justify-center p-4">
      <div className="absolute top-4 right-4">
        <ThemeToggle />
      </div>
      
      <Card className="w-full max-w-md">
        <CardHeader className="text-center">
          <div className="mx-auto w-16 h-16 bg-red-100 dark:bg-red-900/20 rounded-full flex items-center justify-center mb-4">
            <WifiOff className="w-8 h-8 text-red-600 dark:text-red-400" />
          </div>
          <CardTitle className="text-xl font-semibold text-slate-900 dark:text-white">
            Service Unavailable
          </CardTitle>
        </CardHeader>
        <CardContent className="text-center space-y-4">
          <p className="text-slate-600 dark:text-slate-400">
            We're unable to connect to our monitoring service at the moment. 
            This could be due to:
          </p>
          
          <div className="text-left space-y-2 text-sm text-slate-600 dark:text-slate-400">
            <div className="flex items-center gap-2">
              <div className="w-2 h-2 bg-red-500 rounded-full"></div>
              <span>Backend service is down</span>
            </div>
            <div className="flex items-center gap-2">
              <div className="w-2 h-2 bg-red-500 rounded-full"></div>
              <span>Network connectivity issues</span>
            </div>
            <div className="flex items-center gap-2">
              <div className="w-2 h-2 bg-red-500 rounded-full"></div>
              <span>API timeout or error</span>
            </div>
          </div>

          {error && (
            <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-3">
              <div className="flex items-center gap-2 text-red-700 dark:text-red-300">
                <AlertTriangle className="w-4 h-4" />
                <span className="text-sm font-medium">Error Details</span>
              </div>
              <p className="text-xs text-red-600 dark:text-red-400 mt-1 break-all">
                {error}
              </p>
            </div>
          )}

          <div className="pt-4">
            <Button 
              onClick={onRetry} 
              disabled={isLoading}
              className="w-full"
            >
              {isLoading ? (
                <>
                  <RefreshCw className="w-4 h-4 mr-2 animate-spin" />
                  Retrying...
                </>
              ) : (
                <>
                  <Wifi className="w-4 h-4 mr-2" />
                  Try Again
                </>
              )}
            </Button>
          </div>

          <p className="text-xs text-slate-500 dark:text-slate-500">
            If this problem persists, please contact your system administrator.
          </p>
        </CardContent>
      </Card>
    </div>
  );
} 