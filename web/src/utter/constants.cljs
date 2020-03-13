(ns utter.constants)

(defonce ^:const debug? true)

(def login-redirect "https://discordapp.com/api/oauth2/authorize?client_id=633394857682927616&redirect_uri=https%3A%2F%2F6d961e8f.ngrok.io&response_type=code&scope=identify%20email%20guilds")

(defn code-exchange []
  (if debug?
    "http://localhost:8080/login"
    "NO URL YET"))

(defn get-guilds []
  (if debug?
    "http://localhost:8090/guilds"
    "NO URL YET"))

(defn get-guild-details []
  (if debug?
    "http://localhost:8100/guild-details"
    "NO URL YET"))

(defn get-action-name [id]
  (case id
    1 "Ban"
    2 "Unban"
    3 "Kick"
    4 "Mute"
    5 "Unmute"
    6 "Warn"
    7 "Reduced warn"
    8 "Channel Lock"
    9 "Channel Unlock"))