function lobby() {
    console.log("Adding new player to list")
    addPlayer(document.getElementById("player-name").value);
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
 * @param {String} name the name of the player that should be added
 */
function addPlayer(name) {
    addPlayer(name, false);
}

/**
 * Adds the player to the player list
 * @param {string} name the name of the player that should be added
 * @param {boolean} highlighted if true the player will be added highlighted
 */
function addPlayer(name, highlighted) {
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
 * Returns the game code extracted from the URL
 */
function gameCodeFromURL() {
    return window.location.pathname.replace("/lobby/", "");
}

/**
 * Reveals the inner container that contains the player list and the game code
 */
async function revealInnerContainer() {
    document.getElementById("lobby-inner-container").hidden = false;
    document.getElementById("game-code").innerHTML = gameCodeFromURL();
    document.getElementById("game-code").hidden = false;
    document.getElementById("game-code-placeholder").hidden = true;
    // TODO Request to server for list of players
    var response = await fetchData('../api/players_in_game', new Map([["game_code", gameCodeFromURL()]]));
    for (const user of response) {
        addPlayer(user, false);
    }
    document.getElementById("player-list").hidden = false;
    document.getElementById("player-list-placeholder").hidden = true;
}

document.addEventListener("DOMContentLoaded", function(){
    console.info("Initializing page state");
    if (window.location.pathname != '/lobby') {
        console.debug("Initializing page to reflect join game state");
        document.getElementById("enter-player-name").innerHTML = "Join Game";
        revealInnerContainer();
    }
});