package codekata

import java.net.ConnectException
import java.net.URI
import javax.websocket.ClientEndpoint
import javax.websocket.CloseReason
import javax.websocket.ContainerProvider
import javax.websocket.OnClose
import javax.websocket.OnMessage
import javax.websocket.OnOpen
import javax.websocket.Session
import javax.websocket.WebSocketContainer

abstract class GameHandler {
    abstract fun canHandleGame(type: String): Boolean

    abstract fun getMove(type: String, state: List<String>, total_time_ms: Long, time_per_move_ms: Long): String
}

@ClientEndpoint
class ApiClient(val url: String, val apikey: String, val gameHandlers: List<GameHandler>, val debug: Boolean = true) {
    var session: Session? = null

    init {
        try {
            val container: WebSocketContainer = ContainerProvider.getWebSocketContainer()
            container.connectToServer(this, URI(url))
        } catch (e: ConnectException) {
            debug("failed to connect to ${url}")
            throw e
        }
    }

    private fun debug(msg: String) {
        if (debug) {
            println("codekata: $msg")
        }
    }

    @OnOpen
    fun onOpen(session: Session?) {
        this.session = session
        debug("connected to $url")
        // send apikey
        send("version 2")
        send("apikey ${apikey}")
    }

    @OnClose
    fun onClose(@Suppress("UNUSED_PARAMETER") session: Session?, reason: CloseReason?) {
        this.session = null
        debug("disconnected from $url")
        debug("reason: $reason")
    }

    @OnMessage
    fun onMessage(msg: String) {
        if (msg == "okay") {
            debug("okay from server")
        } else if (msg.startsWith("error")) {
            debug("ERROR from server: $msg")
        } else if (msg.startsWith("go")) {
            debug("game from server: $msg")
            // strip leading "go " + parse message
            val content = msg.substring(3)
            val args = content.split(",")

            val id = args[0].trim().toInt()
            val game_type = args[1].trim()
            val total_time = args[2].trim().toLong()
            val time_per_move = args[3].trim().toLong()
            val state = args.subList(4, args.size)

            for (handler in gameHandlers) {
                if (handler.canHandleGame(game_type)) {
                    debug("invoking $game_type handler on game $id with state $state: ...")
                    val move = handler.getMove(game_type, state, total_time, time_per_move)
                    debug("handler returned move $move")
                    send("play $id, $move")
                    return
                }
            }

            debug("couldn't find handle for game type $game_type, will not play")
        } else {
            debug("unrecognized command: $msg")
        }
    }

    fun send(msg: String) {
        session?.getAsyncRemote()?.sendText(msg)
    }
}