import React, { useEffect } from 'react';
import {
  Switch,
  Route,
  useParams,
  useLocation,
  RouteComponentProps,
  withRouter,
} from "react-router-dom";
import { GameId } from './api';
import { ServerConnection } from './App';
import Game from './Game';
import Home from './Home';

interface InnerAppProps {
  conn: ServerConnection;
}

function UrlGame(props: InnerAppProps) {
  const { game_id } = useParams<{game_id: string}>();

  return (
    <Game 
      conn={props.conn} 
      id={+game_id} 
      full={true}
      currentPlayer={props.conn.state.current_user?.id ?? null}
    />
  );
}

function InnerApp(props: InnerAppProps & {gameId: GameId | null, setGameId: (id: GameId | null) => void} & RouteComponentProps) {
  const location = useLocation();
  const query = new URLSearchParams(location.search);
  const {conn, history, gameId, setGameId} = props;
  
  useEffect(() => {
    if(!conn.connected) return;

    // login with apikey if present
    const apikey = query.get("apikey");
    if(apikey !== null) {
      conn.socket?.send(`apikey ${apikey}`);
      conn.socket?.send(`self_user_info`);
    }
  }, 
  // query not included in dependencies: including query forces infinite looping
  // query won't change without the component unmounting + remounting
  [location, conn.socket, conn.connected]);

  // redirect to game
  useEffect(() => {
    if(gameId === null) return;

    history.push(`/game/${gameId}`);
    setGameId(null);
  }, [gameId, setGameId, history]);

  if(!conn.connected) {
    return (
      <div>Connecting to Server...</div>
    );
  }

  return (
    <Switch>
      <Route path="/game/:game_id">
        <UrlGame {...props} />
      </Route>
      <Route exact path="/">
        <Home {...props} />
      </Route>
      <Route path="/">
        Not Found
      </Route>
    </Switch>
  );
}

export default withRouter(InnerApp);