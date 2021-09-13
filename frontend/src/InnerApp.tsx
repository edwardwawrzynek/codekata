import React, { useContext } from 'react';
import { ServerConnection } from './App';
import Game from './Game';

interface InnerAppProps {
  conn: ServerConnection;
}

export default function InnerApp(props: InnerAppProps) {
  
  if(!props.conn.connected) {
    return (
      <div>Connecting to Server...</div>
    );
  }

  return (
    <Game conn={props.conn} id={4} full={true} />
  );
}