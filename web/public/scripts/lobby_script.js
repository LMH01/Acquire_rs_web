let uuid;
let user_name;
let game_code;

function lobby() {
    console.log("Adding new player to list")
    addPlayerHighlighted(document.getElementById("player-name").value);
}

async function demo() {
    var gameCode = await fetchData('api/debug');
    document.getElementById("game-code").innerHTML = gameCode;
    document.getElementById("game-code").hidden = false;
    document.getElementById("game-code-placeholder").hidden = true;
    document.getElementById("lobby-inner-container").hidden = false;
}

/**
 * Adds the player to the player list
 * @param {string} name the name of the player that should be added
 * @param {boolean} highlighted if true the player will be added highlighted
 */
function addPlayerHighlighted(name, highlighted) {
    let div = document.createElement('li');
    if (highlighted) {
        div.className = "list-group-item list-group-item-primary";
    } else {
        div.className = "list-group-item";
    }
    div.innerHTML = name;
    document.getElementById("player-list").append(div);
}

/**
 * Create a new game
 */
async function createGame() {
    if (!usernameEntered()) {
        return;
    }
    let username = document.getElementById("player-name").value;
    let response = await postData("../api/create_game", null, {username: username})
    console.info("Saving uuid and user_name to local storage before redirect");
    localStorage.setItem('uuid', response.uuid);
    localStorage.setItem('user_name', username);
    window.location.href = "/lobby/" + response.game_code;
}

/**
 * Join a game
 */
async function joinGame() {
    if (!usernameEntered()) {
        return;
    }
    let username = document.getElementById("player-name").value;
    let response = await postData("../api/join_game", null, {username: username}, new Map([["game_code", gameCodeFromURL()]]));
    console.log(response);
    if (response == "name_taken") {
        document.getElementById("username-taken-alert").hidden = false;
        return;
    }
    document.getElementById("username-taken-alert").hidden = true;
    window.user_name = username;
    window.uuid = response.uuid;
    window.game_code = response.game_code;
    subscribeEvents(window.uuid);
    reloadPlayerList();
    setJoinedGameComponents();
}

async function leaveGame() {
    if (document.getElementById("leave-game-alert").hidden) {
        document.getElementById("leave-game-alert").hidden = false;
        return;
    }
    let data = await postData("../api/leave_game", window.uuid);
    console.log(window.game_code);
    window.location.href = "/lobby/" + window.game_code;
}

/**
 * Some debug functions to test starting the game
 */
async function startGameDebug() {
    wasm_bindgen.add_player("Rust", false);
   // localStorage.setItem('uuid', window.uuid);
   // localStorage.setItem('user_name', window.user_name);
   // localStorage.setItem('game_code', window.game_code);
   // //window.location.href = "/lobby/" + window.game_code + "/game";
   // window.location.href = "/api/debug/game";
}

/**
 * Dismisses the alert that appears when it is tried to leave the game
 */
function dismissAlerts() {
    document.getElementById("leave-game-alert").hidden = true;
    document.getElementById("username-taken-alert").hidden = true;
}

/**
 * Reveals the inner container that contains the player list and the game code
 */
async function revealInnerContainer() {
    document.getElementById("lobby-inner-container").hidden = false;
    document.getElementById("game-code").innerHTML = gameCodeFromURL();
    document.getElementById("game-code").hidden = false;
    document.getElementById("game-code-placeholder").hidden = true;
    var response = await fetchData('../api/players_in_game', new Map([["game_code", gameCodeFromURL()]]));
    for (const user of response) {
        wasm_bindgen.add_player(user, user == window.user_name);
    }
    document.getElementById("player-list").hidden = false;
    document.getElementById("player-list-placeholder").hidden = true;
}

/**
 * Checks if a username is entered in the `player-name` input field
 * If no name is entered a warning is shown
 */
function usernameEntered() {
    if (document.getElementById("player-name").value == "") {
        alert("Please enter a username");//TODO Maybe make the popup nicer with bootstrap
        return false;
    }
    return true;
}

/**
 * Sets the html components of the page to reflect that the player has joined the game
 */
function setJoinedGameComponents() {
    document.getElementById("join-game").disabled = true;
    document.getElementById("join-game").hidden = true;
    document.getElementById("create-game").hidden = true;
    document.getElementById("leave-game").hidden = false;
    document.getElementById("player-name").value = window.user_name;
    document.getElementById("player-name").disabled = true;
}

/**
 * Reloads the list of joined players
 */
async function reloadPlayerList() {
    console.log("reloading player list");
    var response = await fetchData('../api/players_in_game', new Map([["game_code", gameCodeFromURL()]]));
    document.getElementById("player-list").innerHTML = "";
    for (const user of response) {
        wasm_bindgen.add_player(user, user == window.user_name);
    }
}

/**
 * Subscribes to the event listener at /sse
 */
function subscribeEvents(uuid) {
  function connect() {
    let game_code = gameCodeFromURL();
    let path = game_code + "/" + uuid;
    const events = new EventSource("/sse/" + path);

    events.addEventListener("message", (env) => {
      var data = env.data;
      var msg = JSON.parse(data);
      console.log(msg);
      switch (msg.data[0]) {
        case "AddPlayer":
            wasm_bindgen.add_player(msg.data[1], false);
            break;
        case "ReloadPlayerList":
            reloadPlayerList();
            break;
      }
    });

    events.addEventListener("open", () => {
      console.info(`Connected to event stream at /sse/` + path);
    });

    events.addEventListener("error", () => {
      console.error("connection to event stream at /sse/" + path + " lost");
      console.info("Closing event stream for /sse/" + path);
      events.close();
    });
  }

  connect();
  reloadPlayerList();
}

/**
 * This will initialize the page and add the action to the buttons
 */
function initPage() {
    document.getElementById("create-game").addEventListener('click', createGame);
    document.getElementById("join-game").addEventListener('click', joinGame);
    document.getElementById("leave-game").addEventListener('click', leaveGame);
    document.getElementById("debug").addEventListener('click', startGameDebug);
}

document.addEventListener("DOMContentLoaded", async function(){
    console.info("Initializing page state");
    await wasm_bindgen('../wasm/acquire_rs_wasm_bg.wasm');
    wasm_bindgen.init_lobby();
    initPage();
    if (localStorage.getItem('uuid') != undefined) {
        console.info("Detected local storage, rebuilding page state");
        window.uuid = localStorage.getItem('uuid');
        window.user_name = localStorage.getItem('user_name');
        window.game_code = gameCodeFromURL();
        localStorage.removeItem('uuid');
        localStorage.removeItem('user_name');
        revealInnerContainer();
        setJoinedGameComponents();
        subscribeEvents(window.uuid);
    } else {
        if (window.location.pathname != '/lobby' && window.location.pathname != '/lobby/') {
            console.debug("Initializing page to reflect join game state");
            document.getElementById("create-game").hidden = true;
            document.getElementById("join-game").hidden = false;
            window.game_code = gameCodeFromURL();
            revealInnerContainer();
        }
    }
});