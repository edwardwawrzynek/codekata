import React, { CSSProperties, useCallback, useEffect, useRef, useState } from 'react';
import { Link } from 'react-router-dom';
import { GameId, GamePlayerState, TournamentId, UserId } from './api';
import { ServerConnection } from './App';
import Game, { GameStatus } from './Game';
import ThreeMensMorris from './ThreeMensMorris';

// display a tournament, its players, and constituent games
interface TournamentProps {
  conn: ServerConnection;
  id: TournamentId;
  full: boolean;
  currentPlayer: number | null;
}

export default function Tournament(props: TournamentProps) {
  // observe tournament
  useEffect(() => {
    props.conn.socket?.send(`observe_tournament ${props.id}`);

    return () => {
      props.conn.socket?.send(`stop_observe_tournament ${props.id}`);
    };
  }, [props.id, props.conn.socket]);

  // display loading if we don't have game information yet
  const tournament = props.conn.state.tournaments.get(props.id);

  if(tournament === undefined) {
    return (
      <div>Loading tournament...</div>
    );
  }

  // order games by (active, finished, not started)
  const gameIds = tournament.games.sort((a, b) => {
    const gameA = props.conn.state.games.get(a);
    const gameB = props.conn.state.games.get(b);
    // place unloaded games later
    if(gameA === undefined) {
      return 1;
    }
    if(gameB === undefined) {
      return -1;
    }
    // list started + finished
    if(gameA.started !== gameB.started) {
      return gameA.started ? -1 : 1;
    }
    if(gameA.finished !== gameB.finished) {
      return gameA.finished ? 1 : -1;
    }

    return 0;
  });


  return (
    <div className="tournament">
      <div className="gameId">Tournament #{tournament.id}</div>
      <div className="flex">
        {tournament.players.map((p) => (
          <div className="gamePlayer" style={{background: "#000", color: "#fff"}} key={p.id}>
            <div className="playerName">
              {p.name}
            </div>
            <div className="playerScore">
              {p.wins} - {p.loses} - {p.ties}
            </div>
          </div>
        ))}
      </div>

      <GameStatus
        name="Tournament"
        started={tournament.started}
        finished={tournament.finished}
        winner={tournament.winner}
        getWinnerName={(id) => tournament.players.filter(p => p.id === id)[0].name}
      />

      <div className="tournamentGames">
        {gameIds.map((game_id) => (
          <Game
            conn={props.conn}
            id={game_id}
            full={props.full}
            key={game_id}
            currentPlayer={props.currentPlayer}
          />
        ))}
      </div>
    </div>
  );
}