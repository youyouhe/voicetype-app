import React, { useState, useEffect } from 'react';
import { Server, Zap, BarChart3, ArrowUpRight, AlertCircle } from 'lucide-react';
import { AreaChart, Area, ResponsiveContainer, XAxis, YAxis, Tooltip } from 'recharts';
import { TauriService } from '../../services/tauriService';

interface InfoCardProps {
  icon: React.ReactNode;
  title: string;
  value: string;
  subValue?: React.ReactNode;
  trend?: 'up' | 'down' | 'neutral';
  chart?: boolean;
  chartData?: { time: string; val: number }[];
}

const InfoCard: React.FC<InfoCardProps> = ({ icon, title, value, subValue, chart, chartData }) => (
  <div className="bg-white rounded-xl shadow-sm border border-gray-100 p-5 flex flex-col justify-between h-40 hover:shadow-md transition-shadow">
    <div className="flex justify-between items-start">
      <div className="p-2 bg-gray-50 rounded-lg text-gray-600">
        {icon}
      </div>
      {chart && chartData && (
        <div className="h-10 w-24">
           <ResponsiveContainer width="100%" height="100%">
             <AreaChart data={chartData}>
               <Area type="monotone" dataKey="val" stroke="#3b82f6" fill="#eff6ff" strokeWidth={2} />
             </AreaChart>
           </ResponsiveContainer>
        </div>
      )}
    </div>
    <div className="mt-4">
      <p className="text-sm font-medium text-gray-500">{title}</p>
      <div className="flex items-baseline space-x-2 mt-1">
        <h3 className="text-2xl font-bold text-gray-900">{value}</h3>
        {subValue}
      </div>
    </div>
  </div>
);

export const LiveData: React.FC = () => {
  const [serviceStatus, setServiceStatus] = useState<TauriService.ServiceStatus | null>(null);
  const [latencyData, setLatencyData] = useState<TauriService.LatencyData | null>(null);
  const [usageData, setUsageData] = useState<TauriService.UsageData | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Fetch all live data
  const fetchLiveData = async () => {
    try {
      setError(null);
      const [service, latency, usage] = await Promise.all([
        TauriService.getServiceStatus(),
        TauriService.getLatencyData(),
        TauriService.getUsageData()
      ]);

      setServiceStatus(service);
      setLatencyData(latency);
      setUsageData(usage);
    } catch (err) {
      console.error('Failed to fetch live data:', err);
      setError('Failed to load live data');
    } finally {
      setLoading(false);
    }
  };

  // Initial load and periodic updates
  useEffect(() => {
    fetchLiveData();

    // Update every 10 seconds
    const interval = setInterval(fetchLiveData, 10000);

    return () => clearInterval(interval);
  }, []);

  const getStatusBadge = (status: string) => {
    switch (status) {
      case 'online':
        return <span className="text-xs px-2 py-0.5 rounded-full bg-green-100 text-green-700">Online</span>;
      case 'offline':
        return <span className="text-xs px-2 py-0.5 rounded-full bg-gray-100 text-gray-700">Offline</span>;
      case 'error':
        return <span className="text-xs px-2 py-0.5 rounded-full bg-red-100 text-red-700">Error</span>;
      default:
        return <span className="text-xs px-2 py-0.5 rounded-full bg-gray-100 text-gray-700">Unknown</span>;
    }
  };

  const getTrendIcon = (trend: 'up' | 'down' | 'neutral', value: number) => {
    if (trend === 'up') {
      return <ArrowUpRight className="w-3 h-3 mr-0.5 text-green-500" />;
    } else if (trend === 'down') {
      return <ArrowUpRight className="w-3 h-3 mr-0.5 text-red-500 transform rotate-180" />;
    }
    return null;
  };

  if (loading) {
    return (
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6 w-full">
        {[1, 2, 3].map((i) => (
          <div key={i} className="bg-white rounded-xl shadow-sm border border-gray-100 p-5 flex flex-col justify-between h-40 animate-pulse">
            <div className="flex justify-between items-start">
              <div className="w-9 h-9 bg-gray-200 rounded-lg" />
              {i === 2 && <div className="w-24 h-10 bg-gray-200 rounded" />}
            </div>
            <div className="mt-4">
              <div className="w-20 h-4 bg-gray-200 rounded mb-2" />
              <div className="w-16 h-6 bg-gray-200 rounded" />
            </div>
          </div>
        ))}
      </div>
    );
  }

  if (error) {
    return (
      <div className="bg-red-50 border border-red-200 rounded-xl p-6 w-full">
        <div className="flex items-center space-x-3">
          <AlertCircle className="w-5 h-5 text-red-600" />
          <p className="text-red-800 font-medium">Live data unavailable</p>
        </div>
        <p className="text-red-600 text-sm mt-2">{error}</p>
      </div>
    );
  }

  return (
    <div className="grid grid-cols-1 md:grid-cols-3 gap-6 w-full">
      <InfoCard
        icon={<Server className="w-5 h-5" />}
        title="Active Service"
        value={serviceStatus?.active_service || 'Unknown'}
        subValue={serviceStatus ? getStatusBadge(serviceStatus.status) : <span className="text-xs text-gray-500">Loading...</span>}
      />

      <InfoCard
        icon={<Zap className="w-5 h-5" />}
        title="Avg Latency"
        value={latencyData ? `${latencyData.current}ms` : '--'}
        subValue={
          latencyData ? (
            <span className="text-xs text-gray-400 flex items-center">
              {getTrendIcon(latencyData.trend, latencyData.trend_value)}
              {latencyData.trend_value > 0 ? '+' : ''}{latencyData.trend_value}ms
            </span>
          ) : (
            <span className="text-xs text-gray-500">No data</span>
          )
        }
        chart
        chartData={latencyData?.history}
      />

      <InfoCard
        icon={<BarChart3 className="w-5 h-5" />}
        title="Today's Usage"
        value={usageData ? `${usageData.today_minutes} mins` : '--'}
        subValue={usageData ? (
          <span className="text-xs text-gray-500">
            {usageData.success_rate}% Success ({usageData.successful_requests}/{usageData.total_requests})
          </span>
        ) : (
          <span className="text-xs text-gray-500">No data</span>
        )}
      />
    </div>
  );
};