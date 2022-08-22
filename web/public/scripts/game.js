let user_id;
let user_name;
let game_code;

function generateBoard() {
    for (let i=0; i<=10; i++) {
        addSingleBoardPiece();
    }
}

function addSingleBoardPiece() {
    let div = document.createElement('div');
    div.id = "square";
    div.innerHTML = "A";
    document.getElementById("game-board").append(div);
}

document.addEventListener("DOMContentLoaded", function(){
    //TODO Comment in when page layout is done
    //console.info("Initializing page state");
    //if (localStorage.getItem('user_id') != undefined && localStorage.getItem('user_name') != undefined && localStorage.getItem('game_code') != undefined) {
    //    window.user_id = localStorage.getItem('user_id');
    //    window.user_name = localStorage.getItem('user_name');
    //    window.game_code = localStorage.getItem('game_code');
    //    localStorage.removeItem('user_id');
    //    localStorage.removeItem('user_name');
    //    localStorage.removeItem('game_code');
    //    //TODO Subscribe events should be called here
    //} else {
    //    console.info("Unable to initialize page state, local storage is missing.")
    //    console.info("Redirecting to lobby screen.")
    //    window.location.href = "/lobby/" + gameCodeFromURL();
    //}
    generateBoard();
});