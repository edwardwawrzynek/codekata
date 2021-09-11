import React from 'react';
import SystemStateProvider, { API_URL, SystemContext } from './api';
import InnerApp from './InnerApp';

export default function App() {
  return (
    <SystemStateProvider api_url={API_URL}>
      <InnerApp />
    </SystemStateProvider>
  )
}