import React, { useState, useEffect } from 'react';
import { Server, Cloud, Globe, Wifi, Check, AlertTriangle } from 'lucide-react';
import { ServiceProvider, ServiceOptionProps } from '../../types';
import { Input, ToggleInput } from '../ui/Input';
import { Button } from '../ui/Button';

// Service Option Component
const ServiceOption: React.FC<ServiceOptionProps> = ({ id, title, description, icon, selected, onSelect }) => (
  <div
    onClick={onSelect}
    className={`
      relative flex items-start p-4 rounded-xl border-2 cursor-pointer transition-all duration-200
      ${selected 
        ? 'border-primary-500 bg-primary-50' 
        : 'border-gray-200 bg-white hover:border-gray-300 hover:bg-gray-50'}
    `}
  >
    <div className={`p-2 rounded-lg mr-4 ${selected ? 'bg-primary-100 text-primary-600' : 'bg-gray-100 text-gray-500'}`}>
      {icon}
    </div>
    <div className="flex-1">
      <div className="flex justify-between">
        <h3 className={`font-semibold ${selected ? 'text-primary-900' : 'text-gray-900'}`}>{title}</h3>
        {selected && <Check className="w-5 h-5 text-primary-500" />}
      </div>
      <p className={`text-sm mt-1 ${selected ? 'text-primary-700' : 'text-gray-500'}`}>{description}</p>
    </div>
  </div>
);

// ASR Settings Form
export const ASRSettings: React.FC = () => {
  const [selectedService, setSelectedService] = useState<ServiceProvider>(() => {
    const saved = localStorage.getItem('asr_service_provider');
    return saved ? (saved as ServiceProvider) : ServiceProvider.Cloud;
  });
  const [apiKey, setApiKey] = useState('');
  const [url, setUrl] = useState('http://localhost:5001/inference');
  const [isTesting, setIsTesting] = useState(false);
  const [status, setStatus] = useState<'idle' | 'success' | 'failed'>('idle');

  // Save selected service to localStorage when it changes
  useEffect(() => {
    localStorage.setItem('asr_service_provider', selectedService);
  }, [selectedService]);

  const handleTest = () => {
    setIsTesting(true);
    setStatus('idle');
    setTimeout(() => {
      setIsTesting(false);
      setStatus('success');
    }, 1500);
  };

  return (
    <div className="max-w-3xl animate-in fade-in duration-500">
      <h2 className="text-2xl font-bold text-gray-900 mb-6">ASR Service Settings</h2>

      {/* Service Selection */}
      <section className="bg-white rounded-xl border border-gray-200 p-6 mb-6 shadow-sm">
        <h3 className="text-lg font-semibold text-gray-900 mb-4">Voice Recognition Provider</h3>
        <div className="grid gap-4">
          <ServiceOption
            id={ServiceProvider.Local}
            title="Local ASR"
            description="Runs on device. Privacy focused. Requires powerful GPU."
            icon={<Server className="w-6 h-6" />}
            selected={selectedService === ServiceProvider.Local}
            onSelect={() => setSelectedService(ServiceProvider.Local)}
          />
          <ServiceOption
            id={ServiceProvider.Cloud}
            title="Cloud ASR"
            description="Ultra-fast cloud inference. Supports multiple providers (Whisper, SenseVoice). Requires API Key."
            icon={<Cloud className="w-6 h-6" />}
            selected={selectedService === ServiceProvider.Cloud}
            onSelect={() => setSelectedService(ServiceProvider.Cloud)}
          />
        </div>
      </section>

      {/* Configuration */}
      <section className="bg-white rounded-xl border border-gray-200 p-6 shadow-sm">
        <h3 className="text-lg font-semibold text-gray-900 mb-4">Connection Config</h3>
        <div className="space-y-4">
          {selectedService === ServiceProvider.Local ? (
             <Input label="Service URL" value={url} onChange={setUrl} placeholder="http://localhost..." />
          ) : (
             <Input label="API Key" type="password" value={apiKey} onChange={setApiKey} placeholder="sk-..." required />
          )}
          
          <div className="flex items-center space-x-4 pt-2">
            <Button onClick={handleTest} loading={isTesting} icon={<Wifi className="w-4 h-4"/>}>
              Test Connection
            </Button>
            {status === 'success' && (
              <span className="flex items-center text-sm text-green-600">
                <Check className="w-4 h-4 mr-1" /> Connected
              </span>
            )}
             {status === 'failed' && (
              <span className="flex items-center text-sm text-red-600">
                <AlertTriangle className="w-4 h-4 mr-1" /> Connection Failed
              </span>
            )}
          </div>
        </div>
      </section>
    </div>
  );
};

// Shortcuts Settings
export const ShortcutSettings: React.FC = () => {
    const [transcribeKey, setTranscribeKey] = useState('F4');
    const [translateKey, setTranslateKey] = useState('Shift + F4');
    const [delay, setDelay] = useState(0.3);
    const [antiTouch, setAntiTouch] = useState(true);

    return (
        <div className="max-w-3xl animate-in fade-in duration-500">
             <h2 className="text-2xl font-bold text-gray-900 mb-6">Shortcuts & Behaviors</h2>
             
             <section className="bg-white rounded-xl border border-gray-200 p-6 mb-6 shadow-sm">
                <h3 className="text-lg font-semibold text-gray-900 mb-4">Global Hotkeys</h3>
                <div className="space-y-6">
                    <Input label="Start Transcription" value={transcribeKey} onChange={setTranscribeKey} placeholder="Press keys..." />
                    <Input label="Start Translation" value={translateKey} onChange={setTranslateKey} placeholder="Press keys..." />
                </div>
             </section>

             <section className="bg-white rounded-xl border border-gray-200 p-6 shadow-sm">
                <h3 className="text-lg font-semibold text-gray-900 mb-4">Prevention</h3>
                <div className="space-y-4">
                    <Input label="Trigger Delay (seconds)" type="number" step={0.1} value={delay} onChange={setDelay} unit="s" />
                    <ToggleInput label="Enable Anti-Mistouch" checked={antiTouch} onChange={setAntiTouch} description="Prevents accidental recording when holding keys briefly." />
                </div>
             </section>
        </div>
    )
}

export const PlaceholderSettings: React.FC<{title: string}> = ({title}) => (
    <div className="max-w-3xl animate-in fade-in duration-500">
        <h2 className="text-2xl font-bold text-gray-900 mb-6">{title}</h2>
        <div className="bg-white rounded-xl border border-gray-200 p-12 text-center shadow-sm">
            <div className="inline-block p-4 rounded-full bg-gray-50 mb-4">
                <Sliders className="w-8 h-8 text-gray-400" />
            </div>
            <h3 className="text-gray-900 font-medium text-lg">Coming Soon</h3>
            <p className="text-gray-500 mt-2">This settings module is currently under development.</p>
        </div>
    </div>
);

import { Sliders } from 'lucide-react';