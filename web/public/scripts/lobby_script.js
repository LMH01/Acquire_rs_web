function lobby() {
    console.log("Adding new player to list")
    addPlayer(document.getElementById("player-name").value);
}

function revealPlayers() {
    document.getElementById("player-list").hidden = false;
    document.getElementById("player-list-container-loading").hidden = true;
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
