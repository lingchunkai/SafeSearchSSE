@0xccf4f88a59faca9e;

struct Infoset {
   player @0 :UInt32;
   startSequenceId @1 :UInt32;
   endSequenceId @2 :UInt32;
   parentSequenceId @3 :UInt32;
}

struct Treeplex {
   numSequences @0:UInt32;
   infosets @1 :List(Infoset);
}

struct PayoffMatrix {
   entries @0 :List(PayoffMatrixEntry);
   
   struct PayoffMatrixEntry {
      seqPl1 @0 :UInt32;
      seqPl2 @1 :UInt32;

      payoffPl1 @2 :Float64;
      payoffPl2 @3 :Float64;

      chanceFactor @4 :Float64;
   }
}

struct Game {
   treeplexPl1 @0 :Treeplex;
   treeplexPl2 @1 :Treeplex;

   payoffMatrix @2 :PayoffMatrix;

   subgamesPl1 @3 :List(UInt32);
   subgamesPl2 @4 :List(UInt32);
}
