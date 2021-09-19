import React, { useEffect, useState } from 'react';
import { API_URL, empty_system_state, GameId, SystemState, system_state_run_cmds } from './api';
import InnerApp from './InnerApp';
import './App.css';
import { BrowserRouter } from 'react-router-dom';

export interface ServerConnection {
  socket: WebSocket | null;
  connected: boolean;
  state: SystemState;
}


export default function App() {
  // server connection
  const [sysState, setSysState] = useState<SystemState>(empty_system_state);
  const [socket, setSocket] = useState<WebSocket | null>(null);
  const [connected, setConnected] = useState(false);

  const [gameId, setGameId] = useState<GameId | null>(null);

  // connect to server
  useEffect(() => {
    const newSocket = new WebSocket(API_URL);

    newSocket.addEventListener("open", e => {
      setConnected(true);
      newSocket.send("version 2");
    });

    newSocket.addEventListener("message", e => {
      setSysState((sysState) => system_state_run_cmds(sysState, e.data, (id) => {setGameId(id)}));
    });

    newSocket.addEventListener("error", e => {
      alert("WebSocket error: " + e);
    });

    newSocket.addEventListener("close", e => {
      alert("Connection to server closed");
      setConnected(false);
    });

    setSocket(newSocket);
  }, []);

  return (
    <BrowserRouter>
      <InnerApp conn={{socket, state: sysState, connected}} gameId={gameId} setGameId={setGameId} />
    </BrowserRouter>
  );
}