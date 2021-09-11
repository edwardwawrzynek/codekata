import React, { useContext } from 'react';
import SystemStateProvider, { API_URL, SystemContext } from './api';

export default function InnerApp() {
  let ctx = useContext(SystemContext);
  
  if(ctx.socket === null) {
    return (
      <div>Connecting to Server...</div>
    );
  }

  return (
    <div>Hello</div>
  );
}