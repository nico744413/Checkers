A checkers minimax ai that uses bitboards

features include:
1.AI
2.HUMAN VS HUMAN
3.adjustable search depth (difficulty)

optimizes using bitboards and alpha-beta pruning.
The bitboard is u32 and it corresponds to the board like this:
 
   11  05  31  25 
 10  04  30  24 
   03  29  23  17 
 02  28  22  16 
   27  21  15  09 
 26  20  14  08 
   19  13  07  01 
 18  12  06  00 

Which shifts bits by 7 to go right diagonally, shifts 1 to go left. Minus for down.
