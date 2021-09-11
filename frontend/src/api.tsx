import React, {Fragment, useEffect, useState} from "react";

export const API_URL = process.env.NODE_ENV === 'development' ? 'ws://localhost:9000' : 'ws://codekata.wawrzynek.com';

// Command parsing
type Command = (string | Command)[];

export function parse_command(msg: string): Command[] {
  function parse_nested_args(msg: string, index: number): [Command, number] {
    let res: Command = [];
    let buffer = "";
    
    let i = index;
    while(i < msg.length) {
      const c = msg[i];
      if(c === '[') {
        i++;
        const [cmd, new_i] = parse_nested_args(msg, i);
        res.push(cmd);
        i = new_i - 1;
      } else if(c === ',') {
        if(buffer !== "") {
          res.push(buffer);
        }
        buffer = "";
        i++;
        while(msg[i] === ' ') i++;
        i--;
      } else if(c === ']') {
        if(buffer !== "") {
          res.push(buffer);
        }
        buffer = "";
        i++;
        break;
      } else {
        buffer += c;
      }
      i++;
    }

    if(buffer.length > 0) {
      res.push(buffer);
    }

    return [res, i];
  }

  return msg.trim().split("\n").map((line) => {
    // command id
    if(line.indexOf(' ') === -1) {
      return [line];
    } else {
      const cmd_end_index = line.indexOf(' ');
      const cmd = line.substring(0, cmd_end_index);
      let res: Command = [cmd];

      const args = line.substring(cmd_end_index + 1);
      res = res.concat(parse_nested_args(args, 0)[0]);

      return res;
    }
  })
}

// Game + connection state
// This is a copy of information we got from the server

type GameId = number;
type UserId = number;

// An ongoing game
interface GameState {
  id: GameId;
  type: string;
  owner: UserId;
  started: boolean;
  finished: boolean;
  winner: UserId | "tie" | null;
  dur_total_time: number;
  dur_per_move: number;
  current_player_time: number;
  players: {
    id: UserId,
    score: number,
    time: number
  }[],
  state: string[] | null;
}

function parse_bool(val: string): boolean {
  return val === "true";
}

// parse a "game" command
function game_state_from_cmd(cmd: Command): GameState {
  console.assert(cmd[0] === 'game');

  return {
    id: +cmd[1],
    type: cmd[2] as string,
    owner: +cmd[3],
    started: parse_bool(cmd[4] as string),
    finished: parse_bool(cmd[5] as string),
    winner: cmd[6] === "-" ? null : cmd[6] === "tie" ? "tie" : +cmd[6],
    dur_total_time: +cmd[7],
    dur_per_move: +cmd[8],
    current_player_time: +cmd[9],
    players: (cmd[10] as Command[]).map(p => ({
      id: +p[0],
      score: +p[1],
      time: +p[2],
    })),
    state: cmd[10] === "-" ? null : cmd.slice(10).map(s => s as string),
  };
}

// Collection of games, users, and tournaments that we are interested in
export interface SystemState {
  games: Map<GameId, GameState>;
}

export const empty_system_state: SystemState = { games: new Map() };

// apply a Command to a SystemState, and return the new resulting SystemState
function system_state_run_cmd(state: SystemState, cmd: Command): SystemState {
  if(cmd.length === 0) {
    return state;
  }

  switch(cmd[0]) {
    case 'game':
      const game = game_state_from_cmd(cmd);
      const new_games = new Map(state.games);
      new_games.set(game.id, game);
      return {
        ...state,
        games: new_games
      };
    case 'okay':
      break;
    case 'error':
      alert(`server reports error: ${cmd[1]}`);
      break;
    default:
      alert(`unrecognized command from server: ${cmd}`);
      break;
  }

  return state;
}

// Apply commands to a SystemState, apply the proper updates, and return the resulting SystemState
function system_state_run_cmds(state: SystemState, cmd_msg: string): SystemState {
  let new_state = state;
  parse_command(cmd_msg).forEach((cmd) => {
    new_state = system_state_run_cmd(new_state, cmd);
  });

  return new_state;
}

// a component to provide system state to children + allow children to interact with server
interface SystemStateProviderProps {
  api_url: string;
  children: React.ReactNode
}

interface SystemContextValue {
  socket: WebSocket | null;
  state: SystemState;
}

export const SystemContext = React.createContext<SystemContextValue>({ socket: null, state: empty_system_state });

export default function SystemStateProvider(props: SystemStateProviderProps) {
  // server connection
  const [sysState, setSysState] = useState<SystemState>(empty_system_state);
  const [socket, setSocket] = useState<WebSocket | null>(null);

  // connect to server
  useEffect(() => {
    const newSocket = new WebSocket(props.api_url);

    newSocket.addEventListener("open", e => {
      newSocket.send("version 2");
    });

    newSocket.addEventListener("message", e => {
      setSysState((sysState) => system_state_run_cmds(sysState, e.data));
    });

    newSocket.addEventListener("error", e => {
      alert("WebSocket error: " + e);
    })

    setSocket(newSocket);
  }, [props.api_url]);

  return (
    <SystemContext.Provider value={{socket, state: sysState}}>
      {props.children}
    </SystemContext.Provider>
  );
}