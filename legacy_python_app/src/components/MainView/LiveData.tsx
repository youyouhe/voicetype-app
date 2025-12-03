import React from 'react';
import { Server, Zap, BarChart3, ArrowUpRight } from 'lucide-react';
import { AreaChart, Area, ResponsiveContainer, XAxis, YAxis, Tooltip } from 'recharts';

const data = [
  { time: '10:00', val: 120 },
  { time: '10:05', val: 132 },
  { time: '10:10', val: 101 },
  { time: '10:15', val: 134 },
  { time: '10:20', val: 190 },
  { time: '10:25', val: 230 },
  { time: '10:30', val: 210 },
];

interface InfoCardProps {
  icon: React.ReactNode;
  title: string;
  value: string;
  subValue?: React.ReactNode;
  trend?: 'up' | 'down' | 'neutral';
  chart?: boolean;
}

const InfoCard: React.FC<InfoCardProps> = ({ icon, title, value, subValue, chart }) => (
  <div className="bg-white rounded-xl shadow-sm border border-gray-100 p-5 flex flex-col justify-between h-40 hover:shadow-md transition-shadow">
    <div className="flex justify-between items-start">
      <div className="p-2 bg-gray-50 rounded-lg text-gray-600">
        {icon}
      </div>
      {chart && (
        <div className="h-10 w-24">
           <ResponsiveContainer width="100%" height="100%">
             <AreaChart data={data}>
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
  return (
    <div className="grid grid-cols-1 md:grid-cols-3 gap-6 w-full">
      <InfoCard
        icon={<Server className="w-5 h-5" />}
        title="Active Service"
        value="Groq Whisper"
        subValue={<span className="text-xs px-2 py-0.5 rounded-full bg-green-100 text-green-700">Online</span>}
      />
      
      <InfoCard
        icon={<Zap className="w-5 h-5" />}
        title="Avg Latency"
        value="142ms"
        subValue={<span className="text-xs text-gray-400 flex items-center"><ArrowUpRight className="w-3 h-3 mr-0.5 text-green-500"/> -12ms</span>}
        chart
      />
      
      <InfoCard
        icon={<BarChart3 className="w-5 h-5" />}
        title="Today's Usage"
        value="48 mins"
        subValue={<span className="text-xs text-gray-500">98% Success</span>}
      />
    </div>
  );
};