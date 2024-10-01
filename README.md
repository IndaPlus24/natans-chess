# 🧀 ♕ CHEESS ♔ 🧀 
Documentation time.

# Foreword:
All claims about how the system functions is made with an astonishing lack of testing, meaning that every statement made is to be read as "should".

Sorry.

# Game
The thing that keeps track of the stuff.
## New Game
Use `Game::new()` in order to make new game. You can make your own custom start state.
## Make a Move
Use `self.make_move(from, to)` to make moves.
There exists other functions that move pieces around, and you should not have access to them. If you do, (which you might, due to my incompetence) then please do not use them.
The arguments are self explanatory. However, it is good to know that for every single tuple with two integers, the first one is the column (aka col, file, or x) and the second is the row (aka rank or y). Both are always `u8`.
### Reasons it might fail
If it fails, then it will return false. Here are a few reasons as to why you are doing something stupid:
- The piece can not make that move.
- You are moving a piece that does not exist.
- You are trying to move a piece of the wrong color.
- The move would put the player in check.
- There is at least one piece that needs to be promoted.
Look under Game > Promotion and Piece > Moves for more info on dealing with respective cases.
## Promoting
This uses two functions: `self.get_promotion()` and `self.promote(pos, rank)`. I recommend just putting the output position from the first into the second, along with whatever rank the user picks. Read more about ranks under Piece
There is also `self.get_game_state()`, which will tell you if there are pieces to promote. Read more under Game > Game State
## Game State
Check it with `self.get_game_state()`. There are currently ~~3~~ I mean 4 values:
- `Running`, which means you can play the game normally.
- `Promote`, which means you need to promote some pieces. The state will automatically change when you promote them all. Read more under Game > Promoting.
- `SomethingHasGoneTerriblyWrongMilord`, which is currently unused, because nothing will ever go wrong! (And because I find it funny, and it makes me happy.)
- `CheckMate`. I did not actually plan to include it, but then I just made it test every single move, and it went fast enough, so I am just rolling with it. It does not compute draws, or stalemates, or anything like that. Anyway, the player who currently owns the turn is the player who is in check mate, meaning they are the looser.

I am also going to tell you about `self.get_turn_owner()`, which tells you who is supposed to make a move. This will change when a valid move is made and all pieces are promoted.
## Looking at the Board
This should probably be first, but oh well.
There is just `self.get_piece_at(col, row)`, which gets the piece at that position. Those are explained elsewhere. There is also `self.print_board()` and `self.print_moves(col, row)`, but I do not know why you would ever use them, since they just print stuff to the console. Maybe they could be useful for debugging or something.
Maybe I should add a way to iterate through the pieces.
## Looking at Moves
Since I can not think of why would need to know about every single move, I have only given you a function to look at the possible moves of a piece, with the function `self.get_moves(col, row)`, which gives you the move of the piece at the location (if any), along with their effect. You can read more about what in God's name an "effect" is in the source code.
## Positions
I realize that maybe I should explain how positions work a bit more. Everything is 0 indexed, so they all fall in the range \[0,7]. I already mentioned that every tuple with two `u8` is a position, and that they are always `(col,row)`. You might think I was rather inconsequential with how I decided when to use a tuple or not, and you would right. 
Anyway, something I have not mentioned is that functions like `self.get_moves(col, row)`return the valid move targets in the form of a single `u8`. This is because I thought it would be nice and efficient for the hashmap, but it is not very nice and efficient for you. In order to get the row and column from a `u8` position (`pos`), use `col = pos % 8` and `row = pos >> 3`.
I should have really made it do that automatically or something.
# Piece
Okay, I do not actually know if you can access this type, because I find rust modules weird, and I do not know how to test it. Please let me know if there is something that you need me to fix in order for you to have access to it. You *should* only ever *need* to look at its rank and its color, everything else you can get from the `Game`.
## Ranks
I decided to use the term "rank" to refer to the designations of a piece, like rook, pawn, and knight. Then I found out that people actually use that to refer to the positions on the board, so I felt a bit stupid, but I can not think of a better name, so it sticks.
Basically, each rank is denoted by a character, because at one point I hoped that I would be able to make a command parser, and in that case the character would be useful. But that never happened. Anyway, the ranks are as follows (case sensitive)
- `K` => King
- `Q` => Queen
- `B` => Bishop
- `N` => Knight
- `R` => Rook
- `p` => Pawn

I then found out that people use large and small characters to differentiate between black and white pieces, and that made me feel a bit dumb, because I did not think of that, and I have still not changed it (and I will continue to not do so).
## Build a Piece
The thing that makes Cheess into Cheess is the fact that I am an idiot who decided that it would be fun if I implemented tools for making custom pieces. These tools are needlessly complex, and I do not have the time to explain how it all works. In addition, I do not think there currently exists a way for you to access any of the features needed to make a custom piece. I hope to change all of this sometime soon, but not right now.
# The Other Things
There are a lot of other things going on, but I have no idea if you have access to it, and if you do, you should not need to use it. So if you see anything strange, just ignore it.