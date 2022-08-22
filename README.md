# Acquire_rs_web

This project aims to rewrite my previous [Acquire command line game](https://github.com/LMH01/Acquire_rs) into a webapp.

The web server will be realized by using the the asynchronous web framework [Rocket](https://github.com/SergioBenitez/Rocket).

# Primary Goals

- [ ] Pretty looking game page
- [X] Multiple game instances
- [X] Lobby system with ability to join lobbies by invite code
- [ ] Possibility to reconnect after disconnecting due to internet problems (restoration of game states)
- [ ] Automatic deletion of game instances when all players have disconnected (for cases where the game was not finished)

# Secondary Goals

- [ ] Usage of HTTPS
- [ ] Customizable settings
- [ ] Support for language selection (eng/ger)
- [ ] Responsive design for phone, tablet and desktop
- [ ] Public and private lobbies
- [ ] Kick players from the game lobby
- [ ] Join random public lobbies
- [ ] Extensive documentation using rustdoc

# Maybe sometime
- [ ] Public game browser
