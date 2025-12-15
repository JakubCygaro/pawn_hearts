# Pawn Hearts

A multiplayer chess game.

It requires two players: one as a host and the other as a client.
The client player always plays black pieces and the host plays the white ones.
The game is lost or won when the king of a player is removed from the board by the other player.

There is no check detection so the game will not force you to avoid a loss.

# Building

The build process is quite straight forward on Linux, on Windows I personally recommend using an MSYS rust installation with
clang for mingw64 installed as a package (otherwise you will get a CMake 'LIBCLANG' not found error). I also do not install
cmake through MSYS as it always fucks up every build process I tried.

## Game Assets

By default the game binary will look for a `data` directory located right next to it
if that directory cannot be located the game will fail to start. You can set the environment variable `PH_USE_MEU3=1`
before compiling the game to make the built binary use a [meurglys3](https://github.com/JakubCygaro/meurglys3)
package as a source of assets.

**The assets need to be packaged by you manually**

<img width="930" height="958" alt="image" src="https://github.com/user-attachments/assets/85e819b2-3432-4450-a1bd-bc7c9dca2ff2" />

<img width="1599" height="827" alt="image" src="https://github.com/user-attachments/assets/9f3e1a46-81e5-4fe9-bf1f-9c53c20eeff6" />
