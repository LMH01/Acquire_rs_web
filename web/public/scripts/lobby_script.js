let user_id;
let user_name;

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

async function createOrJoinGame() {
    if (document.getElementById("create-or-join-game").innerHTML = "Create Game") {
        createGame();
    } else {
        joinGame();
    }
}

async function createGame() {
    if (document.getElementById("player-name").value == "") {
        alert("Please enter a username");//TODO Maybe make the popup nicer with bootstrap
        return;
    }
    let username = document.getElementById("player-name").value;
    let response = await postData("../api/create_game", {username: username})
    console.info("Saving user_id and user_name to local storage before redirect");
    localStorage.setItem('user_id', response.user_id);
    localStorage.setItem('user_name', username);
    window.location.href = "/lobby/" + response.game_code;
}

async function joinGame() {

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
        if (user == window.user_name) {
            addPlayer(user, true);
        } else {
            addPlayer(user, false);
        }
    }
    document.getElementById("player-list").hidden = false;
    document.getElementById("player-list-placeholder").hidden = true;
}

document.addEventListener("DOMContentLoaded", function(){
    console.info("Initializing page state");
    if (localStorage.getItem('user_id') != undefined) {
        console.info("Detected local storage, rebuilding page state");
        window.user_id = localStorage.getItem('user_id');
        window.user_name = localStorage.getItem('user_name');
        localStorage.removeItem('user_id');
        localStorage.removeItem('user_name');
        revealInnerContainer();
        document.getElementById("create-or-join-game").innerHTML = "Already Joined";
        document.getElementById("create-or-join-game").className = "btn btn-secondary";
        document.getElementById("create-or-join-game").disabled = true;
        document.getElementById("player-name").value = window.user_name;
        document.getElementById("player-name").disabled = true;
    } else {
        if (window.location.pathname != '/lobby' && window.location.pathname != '/lobby/') {
            console.debug("Initializing page to reflect join game state");
            document.getElementById("create-or-join-game").innerHTML = "Join Game";
            revealInnerContainer();
        }
    }
});