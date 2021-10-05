# Chess Server Protocol

Communication takes place over a websocket connection. The client and server send commands between each other. Commands take the form of:
```
cmd arg0, arg1, arg2, arg3
```

For example, a session between the server and a client that creates, joins, and begins playing a chess game:
```
[client] version 2
[server] okay
[client] apikey f265a80e95fb4734b1557f8d1ff07556
[server] okay
[client] new_game chess
[server] new_game 123
[client] join_game 123
[server] okay
[client] start_game 123
[server] okay
[server] go 123, chess, 341269, 2000, rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
[client] play 123, e2e4
[server] okay
[server] go 123, chess, 305434, 2000, rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq c6 0 2
[client] play 123, g1f3
[server] okay
...
```

## Versioning
There are two versions of the protocol: `1` (backwards compatible with codekata-chess), and `2`. Version `2` should be preferred, as it offers the ability to play multiple games at once and handle games of different types.

New connections start in version `1` by default, so clients must upgrade the connection by sending the command:
```
version 2
```

## Commands
### User commands
|Command|Sender|Description|Protocol Version|
-|-|-|-
|`version <protocol-version>`|Client|Set the protocol version. Accepted versions are `1` or `2`.|
|`error <msg>`|Server|Report an error that occurred in processing a command.|
|`okay`|Server|Report that a command was processed successfully, but no response to the client is needed.|Only reported in version `2` (in version `1`, no response it sent on success).|
|`new_user <name>, <email>, <password>`|Client|Create and log in as a new user.|
|`new_tmp_user <name>`|Client|Create and log in as a new user without an email/password|
|`apikey <key>`|Client|Log in with the given api key.|
|`login <email> <password>`|Client|Log in with an email and password.|
|`logout`|Client|Log out of the logged in user's account.|
|`name <name>`|Client|Set the logged in user's name.|
|`password <pass>`|Client|Set the logged in user's password.|
|`gen_apikey`|Client|Re-generate the logged in user's apikey (server responds with `gen_apikey`).|
|`gen_apikey <key>`|Server|Return the user's generated apikey.|
|`self_user_info`|Client|Get information on the logged in user (server responds with `self_user_info`).|
|`self_user_info <id>, <name>, <email>`|Server|Send information about the current user to the client.|

### Game commands
|Command|Sender|Description|Protocol Version|
-|-|-|-
|`new_game <type>, <total_time>, <time_per_move>`|Client|Create a new game of the given type (server responds with `new_game`). `total_time` is the total time each player gets for the game (in ms), and `time_per_move` is additional time each player is given each move (in ms).|
|`new_game <id>`|Server|Return the new game's id.|
|`new_game_tmp_users <type>, <total_time>, <time_per_move>, <num_users>`|Client|Create a new game with the given number of temporary users, and start that game.|
|`new_game_tmp_users <id>, <user_0_apikey>, <user_1_apikey>, ...`|Server|Return the new game's id and apikeys for each of its players.|
|`observe_game <id>`|Client|Get the state of the game with the given id, and receive updates when that state changes (server responds with `game`).|
|`stop_observe_game <id>`|Client|Stop receiving updates about the state of the game with the given id.|
|`game <id>,<type>,<owning_user_id>,<started>,<finished>,<winner_id OR "tie">,<dur_total_time>,<dur_per_move>,<current_move_start>,<current_player_id>,[[<player0_id>,<player0_name>,<player0_score>,<player0_time>],...],<game_state OR "-">`|Server|Send a game's state to the client.|
|`join_game <id>`|Client|Join the game with the given id. The game must not be started yet.|
|`leave_game <id>`|Client|Leave the game with the given id. The game must not be started yet.|
|`start_game <id>`|Client|Start the game with the given id. The logged in user must own the game.|

### Tournament commands
|Command|Sender|Description|Protocol Version|
-|-|-|-
|`new_tournament <tournament_type>, <game_type>, <total_time>, <time_per_move>, <tournament_options...>`|Client|Create a new tournament. `tournament_options` are dependant on the type of tournament selected. All other options are the same as `new_game`|
|`new_tournament <id>`|Server|Return the new tournament's id.|
|`join_tournament <id>`|Client|Join a tournament with the given id.|
|`leave_tournament <id>`|Client|Leave a tournament with the given id.|
|`start_tournament <id>`|Client|Start a tournament with the given id (you must be owner of the tournament).|
|`observe_tournament <id>`|Client|Get the state of the tournament with the given id and its constituent games, and receive updates when the tournament or constituent games change.|
|`stop_observe_tournament <id>`|Client|Stop getting updates about a tournament and its constituent games.|
|`tournament <id>,<tournament_type>,<owning_user_id>,<game_type>,<started>,<finished>,<winner_id or "tie">,[[<player_0_id>,<name>,<wins>,<loses>,<ties>],[<player_1_id>,<name>,<wins>,<loses>,<ties>],...],<games...>`|Server|Send a tournament's state to a client. The format of `<games>` depends on tournament type.|

### Gameplay Commands
|Command|Sender|Description|Protocol Version|
-|-|-|-
|`go <game_id>, <game_type>, <time_remaining>, <time_for_move>, <game_state>`|Server|Send a game to the client. The client should pick a move to make and send it back with the `play` command. `time_remaining` is the total time (in ms) the client has for the whole game, and `time_for_move` is any additional time the client is given just for this move (in ms).|Version `2` only.|
|`play <game_id>, <move>`|Client|Make a move in the given game. The client should send this in response to a `go` from the server.|Version `2` only.
|`position <game_state>`|Server|Send a game to the client, who should pick a move and respond with the `move` command.|Version `1` only.|
|`move <move>`|Client|Make a move, in response to a `position` command.|Version `1` only.|