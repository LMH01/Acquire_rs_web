# Acquire_rs_web

This project aims to rewrite my previous [Acquire command line game](https://github.com/LMH01/Acquire_rs) into a webapp.

The web server will be realized by using the the asynchronous web framework [Rocket](https://github.com/SergioBenitez/Rocket).

## Building and running
To build and run the server do the following:

1. Clone the repository and cd into the main directory
2. Make sure that the `rust toolchain` and `wasm-pack` are installed
3. Run `./build_wasm.sh`
4. Run `cargo run`

This will start the server which can be accessed under `127.0.0.1:8000`.

## WebAssembly
WebAssembly will be used to write as little JavaScript as possible. The Rust code that is compiled to WebAssembly can be found [here](wasm/).

### Script
The [build_wasm.sh](build_wasm.sh) script is used to build the web assembly parts and copy the output files to `/web/public/wasm`. This way it is easy to update the files once the source code has been modified.

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
