cargo build --release --bin goofspiel
mkdir -p games/goof

echo "Generating deterministic games, general-sum"

target/release/goofspiel -n3 -d2 -o games/goof/goof3d2.game
target/release/goofspiel -n3 -d3 -o games/goof/goof3d3.game

target/release/goofspiel -n4 -d0 -o games/goof/goof4d0.game
target/release/goofspiel -n4 -d1 -o games/goof/goof4d1.game
target/release/goofspiel -n4 -d2 -o games/goof/goof4d2.game
target/release/goofspiel -n4 -d3 -o games/goof/goof4d3.game
target/release/goofspiel -n4 -d4 -o games/goof/goof4d4.game

target/release/goofspiel -n5 -d0 -o games/goof/goof5d0.game
target/release/goofspiel -n5 -d1 -o games/goof/goof5d1.game
target/release/goofspiel -n5 -d2 -o games/goof/goof5d2.game
target/release/goofspiel -n5 -d3 -o games/goof/goof5d3.game
target/release/goofspiel -n5 -d4 -o games/goof/goof5d4.game
target/release/goofspiel -n5 -d5 -o games/goof/goof5d5.game

target/release/goofspiel -n6 -d0 -o games/goof/goof6d0.game
target/release/goofspiel -n6 -d1 -o games/goof/goof6d1.game
target/release/goofspiel -n6 -d2 -o games/goof/goof6d2.game
target/release/goofspiel -n6 -d3 -o games/goof/goof6d3.game
target/release/goofspiel -n6 -d4 -o games/goof/goof6d4.game
target/release/goofspiel -n6 -d5 -o games/goof/goof6d5.game
target/release/goofspiel -n6 -d6 -o games/goof/goof6d6.game

# target/release/goofspiel -n7 -d0 -o games/goof/goof7d0.game
# target/release/goofspiel -n7 -d1 -o games/goof/goof7d1.game
# target/release/goofspiel -n7 -d2 -o games/goof/goof7d2.game
# target/release/goofspiel -n7 -d3 -o games/goof/goof7d3.game
# target/release/goofspiel -n7 -d4 -o games/goof/goof7d4.game
# target/release/goofspiel -n7 -d5 -o games/goof/goof7d5.game
# target/release/goofspiel -n7 -d6 -o games/goof/goof7d6.game
# target/release/goofspiel -n7 -d7 -o games/goof/goof7d7.game

echo "Generating deterministic games, zero-sum"

target/release/goofspiel -n3 -d2 -z -o games/goof/goof3d2z.game
target/release/goofspiel -n3 -d3 -z -o games/goof/goof3d3z.game

target/release/goofspiel -n4 -d0 -z -o games/goof/goof4d0z.game
target/release/goofspiel -n4 -d1 -z -o games/goof/goof4d1z.game
target/release/goofspiel -n4 -d2 -z -o games/goof/goof4d2z.game
target/release/goofspiel -n4 -d3 -z -o games/goof/goof4d3z.game
target/release/goofspiel -n4 -d4 -z -o games/goof/goof4d4z.game

target/release/goofspiel -n5 -d0 -z -o games/goof/goof5d0z.game
target/release/goofspiel -n5 -d1 -z -o games/goof/goof5d1z.game
target/release/goofspiel -n5 -d2 -z -o games/goof/goof5d2z.game
target/release/goofspiel -n5 -d3 -z -o games/goof/goof5d3z.game
target/release/goofspiel -n5 -d4 -z -o games/goof/goof5d4z.game
target/release/goofspiel -n5 -d5 -z -o games/goof/goof5d5z.game

target/release/goofspiel -n6 -d0 -z -o games/goof/goof6d0z.game
target/release/goofspiel -n6 -d1 -z -o games/goof/goof6d1z.game
target/release/goofspiel -n6 -d2 -z -o games/goof/goof6d2z.game
target/release/goofspiel -n6 -d3 -z -o games/goof/goof6d3z.game
target/release/goofspiel -n6 -d4 -z -o games/goof/goof6d4z.game
target/release/goofspiel -n6 -d5 -z -o games/goof/goof6d5z.game
target/release/goofspiel -n6 -d6 -z -o games/goof/goof6d6z.game

# target/release/goofspiel -n7 -d0 -z -o games/goof/goof7d0z.game
# target/release/goofspiel -n7 -d1 -z -o games/goof/goof7d1z.game
# target/release/goofspiel -n7 -d2 -z -o games/goof/goof7d2z.game
# target/release/goofspiel -n7 -d3 -z -o games/goof/goof7d3z.game
# target/release/goofspiel -n7 -d4 -z -o games/goof/goof7d4z.game
# target/release/goofspiel -n7 -d5 -z -o games/goof/goof7d5z.game
# target/release/goofspiel -n7 -d6 -z -o games/goof/goof7d6z.game
# target/release/goofspiel -n7 -d7 -z -o games/goof/goof7d7z.game
echo "Generating stochastic games"

target/release/goofspiel -n3 -d2 -s -o games/goof/goof3d2s.game
target/release/goofspiel -n3 -d3 -s -o games/goof/goof3d3s.game

target/release/goofspiel -n4 -d0 -s -o games/goof/goof4d0s.game
target/release/goofspiel -n4 -d1 -s -o games/goof/goof4d1s.game
target/release/goofspiel -n4 -d2 -s -o games/goof/goof4d2s.game
target/release/goofspiel -n4 -d3 -s -o games/goof/goof4d3s.game
target/release/goofspiel -n4 -d4 -s -o games/goof/goof4d4s.game

target/release/goofspiel -n5 -d0 -s -o games/goof/goof5d0s.game
target/release/goofspiel -n5 -d1 -s -o games/goof/goof5d1s.game
target/release/goofspiel -n5 -d2 -s -o games/goof/goof5d2s.game
target/release/goofspiel -n5 -d3 -s -o games/goof/goof5d3s.game
target/release/goofspiel -n5 -d4 -s -o games/goof/goof5d4s.game
target/release/goofspiel -n5 -d5 -s -o games/goof/goof5d5s.game

# target/release/goofspiel -n6 -d0 -s -o games/goof/goof6d0s.game
# target/release/goofspiel -n6 -d1 -s -o games/goof/goof6d1s.game
# target/release/goofspiel -n6 -d2 -s -o games/goof/goof6d2s.game
# target/release/goofspiel -n6 -d3 -s -o games/goof/goof6d3s.game
# target/release/goofspiel -n6 -d4 -s -o games/goof/goof6d4s.game
# target/release/goofspiel -n6 -d5 -s -o games/goof/goof6d5s.game
# target/release/goofspiel -n6 -d6 -s -o games/goof/goof6d6s.game

echo "Generating stochastic games, zero-sum"

target/release/goofspiel -n3 -d2 -s -z -o games/goof/goof3d2sz.game
target/release/goofspiel -n3 -d3 -s -z -o games/goof/goof3d3sz.game

target/release/goofspiel -n4 -d0 -s -z -o games/goof/goof4d0sz.game
target/release/goofspiel -n4 -d1 -s -z -o games/goof/goof4d1sz.game
target/release/goofspiel -n4 -d2 -s -z -o games/goof/goof4d2sz.game
target/release/goofspiel -n4 -d3 -s -z -o games/goof/goof4d3sz.game
target/release/goofspiel -n4 -d4 -s -z -o games/goof/goof4d4sz.game

target/release/goofspiel -n5 -d0 -s -z -o games/goof/goof5d0sz.game
target/release/goofspiel -n5 -d1 -s -z -o games/goof/goof5d1sz.game
target/release/goofspiel -n5 -d2 -s -z -o games/goof/goof5d2sz.game
target/release/goofspiel -n5 -d3 -s -z -o games/goof/goof5d3sz.game
target/release/goofspiel -n5 -d4 -s -z -o games/goof/goof5d4sz.game
target/release/goofspiel -n5 -d5 -s -z -o games/goof/goof5d5sz.game

# target/release/goofspiel -n6 -d0 -s -z -o games/goof/goof6d0sz.game
# target/release/goofspiel -n6 -d1 -s -z -o games/goof/goof6d1sz.game
# target/release/goofspiel -n6 -d2 -s -z -o games/goof/goof6d2sz.game
# target/release/goofspiel -n6 -d3 -s -z -o games/goof/goof6d3sz.game
# target/release/goofspiel -n6 -d4 -s -z -o games/goof/goof6d4sz.game
# target/release/goofspiel -n6 -d5 -s -z -o games/goof/goof6d5sz.game
# target/release/goofspiel -n6 -d6 -s -z -o games/goof/goof6d6sz.game
