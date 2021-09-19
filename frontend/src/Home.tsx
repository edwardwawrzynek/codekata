import React, { useCallback } from 'react';
import { ServerConnection } from './App';

export default function Home(props: {conn: ServerConnection}) {
  const {conn} = props;

  const newGame = useCallback(() => {
    if(!conn.connected) return;

    conn.socket?.send(`new_game_tmp_users nine_holes, 100000000, 0, 2`);
  }, [conn]);

  return (
    <div>
      <h1>Codekata</h1>
      <button onClick={newGame}>Create New Game</button>
    </div>
  );
}