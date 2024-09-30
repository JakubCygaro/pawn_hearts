# TODO

- special moves like that one where a pawn can move two cells at the start
- look into how I could easily flip the whole board upside down so the black player sees it
with his pieces at the bottom (reversing the iterator in the draw method does the job render-wise, but I also have to 'flip' the selection logic (which could probably be done by fliping the coordinates of the mouse/point-on-rectangle before doing the selection logic))
- start adding basic gameplay logic, player turns, maybe time optional constraints for player turns so that it can easily be integrated with the netcode 
- write the actual network logic (maybe like a state machine? where the game either does client side move logic and passes it to the peer or waits for a response from the peer and applies the move on its side)

Most of the web logic can be done by just passing the moves since each game instance validates its moves before even doing them. So theoretically a host does not need to validate their move since it's already been validated by the peer. But the protocol should enforce bilateral validations which are either accepted by the server or forbidden by it. That way a non-host has to wait for a response from the server that tells it whether the move is valid.
