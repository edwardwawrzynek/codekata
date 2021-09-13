import React, { CSSProperties, useContext, useEffect, useRef, useState } from 'react';
import { GameId, GamePlayerState, UserId } from './api';
import { ServerConnection } from './App';
import NineHoles from './NineHoles';

// convert a time in ms to string
function msToStr(ms: number): string {
  return `${(ms / 1000).toFixed(1)} s`;
}

// game state display component type
export interface GameComponentProps {
  full: boolean;
  type: string;
  state: string[];
  gameCfg: GameType;
}
type GameComponentType = React.ComponentType<GameComponentProps>;


interface PlayerColors {
  fg: string;
  bg: string;
  border?: string;
  active?: string;
}

// A game component for unknown game type
function UnknownGameType(props: GameComponentProps) {
  if(props.full) {
    return (
      <div>Unknown game type <strong>{props.type}</strong> with state {props.state.join(', ')}</div>
    );
  } else {
    return (
      <div>Unknown game type {props.type}</div>
    );
  }
}

// game types + configuration for them
interface GameType {
  playerColors: PlayerColors[];
  doScore: boolean;
  component: GameComponentType;
}

const gameTypeMap: {[game_type: string]: GameType} = {
  "chess": {
    playerColors: [{fg: "#000", bg: "#fff", border: "#000"}, {fg: "#fff", bg: "#000"}],
    doScore: false,
    component: UnknownGameType,
  },
  "nine_holes": {
    playerColors: [{fg: "#fff", bg: "#253e69", active: "white"}, {fg: "#fff", bg: "#851229", active: "white"}],
    doScore: false,
    component: NineHoles,
  }
};

const defaultGameType: GameType = {
  playerColors: [{fg: "#000", bg: "#fff", border: "#000"}],
  doScore: true,
  component: UnknownGameType
};

// display a game player's information

function gamePlayerName(player: GamePlayerState) {
  return `Player #${player.id}`;
}

function GamePlayer(props: GamePlayerState & {full: boolean, color: PlayerColors, doScore: boolean, isCurrent: boolean, currentMoveStart: Date, timePerPlayer: number}) {
  const styles = {
    "--player-fg": props.color.fg, 
    "--player-bg": props.color.bg,
    "--player-border": props.color.border ?? props.color.bg,
    "--player-active": props.color.active ?? "#ff0000",
  } as CSSProperties;

  const [totalTimeLeft, setTotalTimeLeft] = useState(props.time);
  const [currentTimeLeft, setCurrentTimeLeft] = useState(props.timePerPlayer);

  const callbackId = useRef<number | null>(null);

  useEffect(() => {
    callbackId.current = window.setInterval(() => {
      if(props.isCurrent) {
        const elapsed = new Date().getTime() - props.currentMoveStart.getTime();
        setCurrentTimeLeft(Math.max(props.timePerPlayer - elapsed, 0.0));
        const totalLost = Math.max(elapsed - props.timePerPlayer, 0);
        setTotalTimeLeft(Math.max(props.time - totalLost, 0.0));
      }
    }, 100);

    return () => {
      if(callbackId.current !== null) {
        window.clearInterval(callbackId.current);
      }
    }
  }, [props.isCurrent, props.currentMoveStart, setCurrentTimeLeft, setTotalTimeLeft, props.time, props.timePerPlayer]);

  return (
    <div className={`gamePlayer ${props.isCurrent ? 'gamePlayerActive' : ''}`} style={styles}>
      <div className="playerName">
        {gamePlayerName(props)}
      </div>
      {props.full &&
        <div className="flex">
          <div className="playerTime">
            {msToStr(totalTimeLeft)}
            {props.isCurrent &&
              `+ ${msToStr(currentTimeLeft)}`
            }
          </div>
          {props.doScore &&
            <div className="playerScore">
              Score: {props.score}
            </div>
          }
        </div>
      }
    </div>
  );
}

// Display a game, its players, and its state

interface GameProps {
  conn: ServerConnection;
  id: GameId;
  full: boolean;
}

export default function Game(props: GameProps) {
  // observe this game
  useEffect(() => {
    props.conn.socket?.send(`observe_game ${props.id}`);

    return () => {
      props.conn.socket?.send(`stop_observe_game ${props.id}`);
    };
  }, [props.id, props.conn.socket]);

  // display loading if we don't have game information yet
  const game = props.conn.state.games.get(props.id);
  console.log(game);
  if(game === undefined) {
    return (
      <div>Loading game...</div>
    );
  }
  const gameType = gameTypeMap[game.type] ?? defaultGameType;
  const GameComponent = gameType.component;

  return (
    <div className="game">
      <div className="flex">
        {game.players.map((p, i) => 
          <GamePlayer
            key={i}
            {...p} 
            full={props.full} 
            color={gameType.playerColors[i] ?? gameType.playerColors[0]}
            doScore={gameType.doScore}
            isCurrent={p.id === game.current_player}
            currentMoveStart={game.current_move_start}
            timePerPlayer={game.dur_per_move}
          />  
        )}
      </div>

      <div className="gameStatus">
        {game.started ? (game.finished ? "" : "Game In Progress") : "Game Not Started"}
        {game.finished &&
          <span className="gameResult">
            {
              game.winner === null ? "No Result" : 
              game.winner === "tie" ? "Tie" : 
              `${gamePlayerName(game.players.filter(p => p.id === game?.winner)[0])} Wins`
            }
          </span>
        }
      </div>

      {game.state !== null &&
        <GameComponent state={game.state} type={game.type} full={props.full} gameCfg={gameType} />
      }
    </div>
  );
}