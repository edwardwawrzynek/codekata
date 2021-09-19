package codekata.three_mens_morris

import codekata.GameHandler

// A cell on a three men's morris board.
enum class Cell {
    Empty,
    EnemyPiece,
    OurPiece
}

// A player in a three men's morris game.
enum class Player {
    Us {
        override fun toCell() = Cell.OurPiece
        override fun other() = Enemy
    },
    Enemy {
        override fun toCell() = Cell.EnemyPiece
        override fun other() = Us
    };

    abstract fun toCell(): Cell
    abstract fun other(): Player
}

// A move in a three men's morris game
abstract class Move {
    abstract fun applyToBoard(board: Board): Board
}

// A move that places a piece
class MovePiecePlace(val x: Int, val y: Int, val player: Player) : Move() {
    override fun applyToBoard(board: Board): Board {
        val cells = board.copyCells()
        // make move
        cells[y][x] = player.toCell()
        return Board(cells, player.other())
    }

    override fun toString() = "$x $y"
}

// A move that moves a piece
class MovePieceMove(val srcX: Int, val srcY: Int, val dstX: Int, val dstY: Int, val player: Player) : Move() {
    override fun applyToBoard(board: Board): Board {
        val cells = board.copyCells()
        // make move
        cells[srcY][srcX] = Cell.Empty
        cells[dstY][dstX] = player.toCell()
        return Board(cells, player.other())
    }

    override fun toString() = "$srcX $srcY $dstX $dstY"
}

// A three men's morris board. The board is a 3 x 3 matrix of cells.
// Each cell is either empty, contains an opponent piece, or contains one of our pieces.
class Board(val cells: Array<Array<Cell>>, val currentPlayer: Player) {
    companion object {
        // Create a board from a state string sent by the server
        fun fromString(state: List<String>): Board {
            val player = state[1].trim()[0]
            val cells = Array(3) { Array(3) { Cell.Empty } }
            val board = state[0].trim()

            var i = 0
            for (y in 0 until 3) {
                for (x in 0 until 3) {
                    val cell = board[i++]

                    if (cell == player) {
                        cells[y][x] = Cell.OurPiece
                    } else if (cell == '.') {
                        cells[y][x] = Cell.Empty
                    } else {
                        cells[y][x] = Cell.EnemyPiece
                    }
                }
            }

            return Board(cells, Player.Us)
        }
    }

    // Return the number of pieces the given player has yet to place
    fun piecesRemainng(player: Player): Int {
        return 3 - cells.sumOf { row -> row.count { cell -> cell == player.toCell() } }
    }

    // Check if a player has won
    fun playerWon(player: Player): Boolean {
        // check horizontals
        val wonRow = cells.any { row -> row.all { cell -> cell == player.toCell() } }
        // check verticals
        val wonCol = (0 until 3).any { x -> (0 until 3).all { y -> cells[y][x] == player.toCell() } }
        // check diagonals
        val wonDiag0 = (0 until 3).all { i -> cells[i][i] == player.toCell() }
        val wonDiag1 = (0 until 3).all { i -> cells[2 - i][i] == player.toCell() }

        return wonRow || wonCol || wonDiag0 || wonDiag1
    }

    // Return the winning player, or null if no player has won
    fun winner(): Player? {
        return if (playerWon(Player.Us)) {
            Player.Us
        } else if (playerWon(Player.Enemy)) {
            Player.Enemy
        } else {
            null
        }
    }

    // Get the cell at the given x, y
    fun getCell(x: Int, y: Int): Cell {
        return cells[y][x]
    }

    // Return a copy of this board's cells
    fun copyCells(): Array<Array<Cell>> {
        // copy board
        val newCells = Array(3) { Array(3) { Cell.Empty } }
        for (y in 0 until 3) {
            for (x in 0 until 3) {
                newCells[y][x] = cells[y][x]
            }
        }
        return newCells
    }

    // Return the state of this board after move has been applied to it.
    // this is not changed (the new board is returned)
    fun applyMove(move: Move): Board {
        return move.applyToBoard(this)
    }

    // Get all legal moves that can be made on this board
    fun legalMoves(): List<Move> {
        val remaining = piecesRemainng(currentPlayer)
        // if we have pieces left, we need to place those pieces
        if (remaining > 0) {
            val moves = mutableListOf<Move>()
            for (y in 0 until 3) {
                for (x in 0 until 3) {
                    if (cells[y][x] == Cell.Empty) {
                        moves.add(MovePiecePlace(x, y, currentPlayer))
                    }
                }
            }
            return moves
        }
        // otherwise, we need to move our existing pieces
        else {
            val moves = mutableListOf<Move>()
            for (srcY in 0 until 3) {
                for (srcX in 0 until 3) {
                    if (cells[srcY][srcX] != currentPlayer.toCell()) continue

                    for (dstY in 0 until 3) {
                        for (dstX in 0 until 3) {
                            if (cells[dstY][dstX] == Cell.Empty) {
                                moves.add(MovePieceMove(srcX, srcY, dstX, dstY, currentPlayer))
                            }
                        }
                    }
                }
            }
            return moves
        }
    }

    override fun toString(): String {
        var res = ""
        for (row in cells) {
            for (cell in row) {
                res += when (cell) {
                    Cell.Empty -> ". "
                    Cell.OurPiece -> "U "
                    Cell.EnemyPiece -> "E "
                }
            }
            res += "\n"
        }
        val player = when (currentPlayer) {
            Player.Us -> "Us"
            Player.Enemy -> "Enemy"
        }
        res += "current player: $player"
        return res
    }
}

// A handler for a three men's morris game.
// onMove is called when a move is required.
class ThreeMensMorrisHandler(val onMove: (board: Board, total_time_ms: Long, time_per_move_ms: Long) -> Move) :
    GameHandler() {
    override fun canHandleGame(type: String) = type == "three_mens_morris"

    override fun getMove(type: String, state: List<String>, total_time_ms: Long, time_per_move_ms: Long): String {
        println("three_mens_morris handler: time ${total_time_ms / 1000} s + ${time_per_move_ms / 1000} s, board:")
        val board = Board.fromString(state)
        println(board)
        println("three_mens_morris handler: invoking user logic to find best move: ...")
        val move = onMove(board, total_time_ms, time_per_move_ms)
        return move.toString()
    }
}