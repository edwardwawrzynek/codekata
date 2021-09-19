import codekata.three_mens_morris.Board
import codekata.three_mens_morris.Move
import codekata.three_mens_morris.ThreeMensMorrisHandler
import codekata.three_mens_morris.Player
import kotlin.system.exitProcess

object Sample {
    // CHANGE THIS CODE
    // You should implement your client logic here.
    // The current program is just an example, and should be replaced with your program
    fun pickMove(board: Board, total_time_ms: Long, time_per_move_ms: Long): Move {
        // get all legal moves
        val moves = board.legalMoves()
        // if any move lets us win, make that move
        for (move in moves) {
            // apply the move to the board (this returns a new board with the move made)
            val newBoard = board.applyMove(move)
            // check if we have won this new board
            if (newBoard.winner() == Player.Us) {
                // this is a winning move, so we want to make it
                return move
            }
        }
        // no move lets us win, so filter out any that let the enemy win
        val goodMoves = moves.filter { move ->
            // check if enemy can win if we make this win
            val newBoard = board.applyMove(move)
            !newBoard.legalMoves().any { enemyMove ->
                newBoard.applyMove(enemyMove).winner() == Player.Enemy
            }
        }

        // if we have no good moves, then pick from all moves at random
        if (goodMoves.isEmpty()) {
            return moves[0]
        }
        // otherwise, pick at random from good moves
        else {
            return goodMoves.random()
        }
    }

    @JvmStatic
    fun main(args: Array<String>) {
        if (args.size < 2) {
            println("ARGS: server_url apikey")
            println("Example (running through gradle):")
            println("./gradlew run --args \"ws://codekata.wawrzynek.com API_KEY\"")
            exitProcess(1)
        }

        codekata.ApiClient(args[0], args[1], listOf(
            ThreeMensMorrisHandler { board, total_time, time_per_move -> pickMove(board, total_time, time_per_move) }
        ))
        while (true) {
            Thread.sleep(100000)
        }
    }


}