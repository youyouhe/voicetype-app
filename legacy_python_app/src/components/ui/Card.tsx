import React from 'react';
import { motion } from 'framer-motion';
import { LucideIcon } from 'lucide-react';

interface CardProps {
  children: React.ReactNode;
  className?: string;
  hover?: boolean;
  icon?: LucideIcon;
  title?: string;
  description?: string;
  value?: string | number;
  trend?: 'up' | 'down' | 'neutral';
}

export const Card: React.FC<CardProps> = ({
  children,
  className = '',
  hover = false,
  icon: Icon,
  title,
  description,
  value,
  trend = 'neutral'
}) => {
  return (
    <motion.div
      whileHover={hover ? { scale: 1.02, y: -2 } : { scale: 1, y: 0 }}
      className={`
        bg-white rounded-xl shadow-lg border border-white p-6
        ${hover ? 'hover:shadow-xl' : ''}
        ${className}
      `}
      transition={{ duration: 0.2, ease: "easeOut" }}
    >
      {title && (
        <div className="flex items-center justify-between mb-2">
          <h3 className="text-lg font-semibold text-gray-900">{title}</h3>
          {icon && (
            <Icon className="w-5 h-5 text-gray-400" />
          )}
        </div>
      )}
      
      {value !== undefined && (
        <div className="flex items-center space-x-2 mb-2">
          <span className="text-2xl font-bold text-gray-900">{value}</span>
          {trend !== 'neutral' && (
            <motion.div
              className={`
                w-4 h-4 rounded-full
                ${trend === 'up' ? 'bg-green-100' : 'bg-red-100'}
              `}
              animate={{ rotate: trend === 'up' ? -45 : 45 }}
            >
              <div className={`
                w-2 h-2 rounded-full
                ${trend === 'up' ? 'bg-green-500' : 'bg-red-500'}
                translate-y-[-1px]
              `} />
            </motion.div>
          )}
        </div>
      )}

      {description && (
        <p className="text-sm text-gray-600 mb-4">{description}</p>
      )}

      {children}
    </motion.div>
  );
};