cargo build --release --bin leduc
mkdir -p games/leduc

echo "Generating standard games p=1 b=2,4 r=0.01 (general-sum)", with subgames

target/release/leduc -m1 -n2 -p1 -b "2,4" -r 0.01 -s second_round -o games/leduc/leduc_m1n2p1b24r0.01.game
target/release/leduc -m2 -n2 -p1 -b "2,4" -r 0.01 -s second_round -o games/leduc/leduc_m2n2p1b24r0.01.game
target/release/leduc -m3 -n2 -p1 -b "2,4" -r 0.01 -s second_round -o games/leduc/leduc_m3n2p1b24r0.01.game
target/release/leduc -m4 -n2 -p1 -b "2,4" -r 0.01 -s second_round -o games/leduc/leduc_m4n2p1b24r0.01.game
target/release/leduc -m5 -n2 -p1 -b "2,4" -r 0.01 -s second_round -o games/leduc/leduc_m5n2p1b24r0.01.game

target/release/leduc -m1 -n3 -p1 -b "2,4" -r 0.01 -s second_round -o games/leduc/leduc_m1n3p1b24r0.01.game
target/release/leduc -m2 -n3 -p1 -b "2,4" -r 0.01 -s second_round -o games/leduc/leduc_m2n3p1b24r0.01.game
target/release/leduc -m3 -n3 -p1 -b "2,4" -r 0.01 -s second_round -o games/leduc/leduc_m3n3p1b24r0.01.game
target/release/leduc -m4 -n3 -p1 -b "2,4" -r 0.01 -s second_round -o games/leduc/leduc_m4n3p1b24r0.01.game
target/release/leduc -m5 -n3 -p1 -b "2,4" -r 0.01 -s second_round -o games/leduc/leduc_m5n3p1b24r0.01.game

target/release/leduc -m1 -n4 -p1 -b "2,4" -r 0.01 -s second_round -o games/leduc/leduc_m1n4p1b24r0.01.game
target/release/leduc -m2 -n4 -p1 -b "2,4" -r 0.01 -s second_round -o games/leduc/leduc_m2n4p1b24r0.01.game
target/release/leduc -m3 -n4 -p1 -b "2,4" -r 0.01 -s second_round -o games/leduc/leduc_m3n4p1b24r0.01.game
target/release/leduc -m4 -n4 -p1 -b "2,4" -r 0.01 -s second_round -o games/leduc/leduc_m4n4p1b24r0.01.game
target/release/leduc -m5 -n4 -p1 -b "2,4" -r 0.01 -s second_round -o games/leduc/leduc_m5n4p1b24r0.01.game

target/release/leduc -m1 -n5 -p1 -b "2,4" -r 0.01 -s second_round -o games/leduc/leduc_m1n5p1b24r0.01.game
target/release/leduc -m2 -n5 -p1 -b "2,4" -r 0.01 -s second_round -o games/leduc/leduc_m2n5p1b24r0.01.game
target/release/leduc -m3 -n5 -p1 -b "2,4" -r 0.01 -s second_round -o games/leduc/leduc_m3n5p1b24r0.01.game
target/release/leduc -m4 -n5 -p1 -b "2,4" -r 0.01 -s second_round -o games/leduc/leduc_m4n5p1b24r0.01.game
target/release/leduc -m5 -n5 -p1 -b "2,4" -r 0.01 -s second_round -o games/leduc/leduc_m5n5p1b24r0.01.game

target/release/leduc -m1 -n10 -p1 -b "2,4" -r 0.01 -s second_round -o games/leduc/leduc_m1n10p1b24r0.01.game
target/release/leduc -m2 -n10 -p1 -b "2,4" -r 0.01 -s second_round -o games/leduc/leduc_m2n10p1b24r0.01.game
target/release/leduc -m3 -n10 -p1 -b "2,4" -r 0.01 -s second_round -o games/leduc/leduc_m3n10p1b24r0.01.game
target/release/leduc -m4 -n10 -p1 -b "2,4" -r 0.01 -s second_round -o games/leduc/leduc_m4n10p1b24r0.01.game
target/release/leduc -m5 -n10 -p1 -b "2,4" -r 0.01 -s second_round -o games/leduc/leduc_m5n10p1b24r0.01.game

target/release/leduc -m1 -n20 -p1 -b "2,4" -r 0.01 -s second_round -o games/leduc/leduc_m1n20p1b24r0.01.game
target/release/leduc -m2 -n20 -p1 -b "2,4" -r 0.01 -s second_round -o games/leduc/leduc_m2n20p1b24r0.01.game
target/release/leduc -m3 -n20 -p1 -b "2,4" -r 0.01 -s second_round -o games/leduc/leduc_m3n20p1b24r0.01.game
target/release/leduc -m4 -n20 -p1 -b "2,4" -r 0.01 -s second_round -o games/leduc/leduc_m4n20p1b24r0.01.game
target/release/leduc -m5 -n20 -p1 -b "2,4" -r 0.01 -s second_round -o games/leduc/leduc_m5n20p1b24r0.01.game

echo "Generating standard games p=1 b=2,4 r=0.01 (general-sum)", with subgames, 2rd action

target/release/leduc -m1 -n2 -p1 -b "2,4" -r 0.01 -s 2th_action -o games/leduc/leduc_m1n2p1b24r0.01_action2.game
target/release/leduc -m2 -n2 -p1 -b "2,4" -r 0.01 -s 2th_action -o games/leduc/leduc_m2n2p1b24r0.01_action2.game
target/release/leduc -m3 -n2 -p1 -b "2,4" -r 0.01 -s 2th_action -o games/leduc/leduc_m3n2p1b24r0.01_action2.game
target/release/leduc -m4 -n2 -p1 -b "2,4" -r 0.01 -s 2th_action -o games/leduc/leduc_m4n2p1b24r0.01_action2.game
target/release/leduc -m5 -n2 -p1 -b "2,4" -r 0.01 -s 2th_action -o games/leduc/leduc_m5n2p1b24r0.01_action2.game

target/release/leduc -m1 -n3 -p1 -b "2,4" -r 0.01 -s 2th_action -o games/leduc/leduc_m1n3p1b24r0.01_action2.game
target/release/leduc -m2 -n3 -p1 -b "2,4" -r 0.01 -s 2th_action -o games/leduc/leduc_m2n3p1b24r0.01_action2.game
target/release/leduc -m3 -n3 -p1 -b "2,4" -r 0.01 -s 2th_action -o games/leduc/leduc_m3n3p1b24r0.01_action2.game
target/release/leduc -m4 -n3 -p1 -b "2,4" -r 0.01 -s 2th_action -o games/leduc/leduc_m4n3p1b24r0.01_action2.game
target/release/leduc -m5 -n3 -p1 -b "2,4" -r 0.01 -s 2th_action -o games/leduc/leduc_m5n3p1b24r0.01_action2.game

target/release/leduc -m1 -n4 -p1 -b "2,4" -r 0.01 -s 2th_action -o games/leduc/leduc_m1n4p1b24r0.01_action2.game
target/release/leduc -m2 -n4 -p1 -b "2,4" -r 0.01 -s 2th_action -o games/leduc/leduc_m2n4p1b24r0.01_action2.game
target/release/leduc -m3 -n4 -p1 -b "2,4" -r 0.01 -s 2th_action -o games/leduc/leduc_m3n4p1b24r0.01_action2.game
target/release/leduc -m4 -n4 -p1 -b "2,4" -r 0.01 -s 2th_action -o games/leduc/leduc_m4n4p1b24r0.01_action2.game
target/release/leduc -m5 -n4 -p1 -b "2,4" -r 0.01 -s 2th_action -o games/leduc/leduc_m5n4p1b24r0.01_action2.game

target/release/leduc -m1 -n5 -p1 -b "2,4" -r 0.01 -s 2th_action -o games/leduc/leduc_m1n5p1b24r0.01_action2.game
target/release/leduc -m2 -n5 -p1 -b "2,4" -r 0.01 -s 2th_action -o games/leduc/leduc_m2n5p1b24r0.01_action2.game
target/release/leduc -m3 -n5 -p1 -b "2,4" -r 0.01 -s 2th_action -o games/leduc/leduc_m3n5p1b24r0.01_action2.game
target/release/leduc -m4 -n5 -p1 -b "2,4" -r 0.01 -s 2th_action -o games/leduc/leduc_m4n5p1b24r0.01_action2.game
target/release/leduc -m5 -n5 -p1 -b "2,4" -r 0.01 -s 2th_action -o games/leduc/leduc_m5n5p1b24r0.01_action2.game

target/release/leduc -m1 -n10 -p1 -b "2,4" -r 0.01 -s 2th_action -o games/leduc/leduc_m1n10p1b24r0.01_action2.game
target/release/leduc -m2 -n10 -p1 -b "2,4" -r 0.01 -s 2th_action -o games/leduc/leduc_m2n10p1b24r0.01_action2.game
target/release/leduc -m3 -n10 -p1 -b "2,4" -r 0.01 -s 2th_action -o games/leduc/leduc_m3n10p1b24r0.01_action2.game
target/release/leduc -m4 -n10 -p1 -b "2,4" -r 0.01 -s 2th_action -o games/leduc/leduc_m4n10p1b24r0.01_action2.game
target/release/leduc -m5 -n10 -p1 -b "2,4" -r 0.01 -s 2th_action -o games/leduc/leduc_m5n10p1b24r0.01_action2.game

target/release/leduc -m1 -n20 -p1 -b "2,4" -r 0.01 -s 2th_action -o games/leduc/leduc_m1n20p1b24r0.01_action2.game
target/release/leduc -m2 -n20 -p1 -b "2,4" -r 0.01 -s 2th_action -o games/leduc/leduc_m2n20p1b24r0.01_action2.game
target/release/leduc -m3 -n20 -p1 -b "2,4" -r 0.01 -s 2th_action -o games/leduc/leduc_m3n20p1b24r0.01_action2.game
target/release/leduc -m4 -n20 -p1 -b "2,4" -r 0.01 -s 2th_action -o games/leduc/leduc_m4n20p1b24r0.01_action2.game
target/release/leduc -m5 -n20 -p1 -b "2,4" -r 0.01 -s 2th_action -o games/leduc/leduc_m5n20p1b24r0.01_action2.game

echo "Generating standard games p=1 b=2,4 r=0.01 (general-sum)", with subgames, 3rd action

target/release/leduc -m1 -n2 -p1 -b "2,4" -r 0.01 -s 3th_action -o games/leduc/leduc_m1n2p1b24r0.01_action3.game
target/release/leduc -m2 -n2 -p1 -b "2,4" -r 0.01 -s 3th_action -o games/leduc/leduc_m2n2p1b24r0.01_action3.game
target/release/leduc -m3 -n2 -p1 -b "2,4" -r 0.01 -s 3th_action -o games/leduc/leduc_m3n2p1b24r0.01_action3.game
target/release/leduc -m4 -n2 -p1 -b "2,4" -r 0.01 -s 3th_action -o games/leduc/leduc_m4n2p1b24r0.01_action3.game
target/release/leduc -m5 -n2 -p1 -b "2,4" -r 0.01 -s 3th_action -o games/leduc/leduc_m5n2p1b24r0.01_action3.game

target/release/leduc -m1 -n3 -p1 -b "2,4" -r 0.01 -s 3th_action -o games/leduc/leduc_m1n3p1b24r0.01_action3.game
target/release/leduc -m2 -n3 -p1 -b "2,4" -r 0.01 -s 3th_action -o games/leduc/leduc_m2n3p1b24r0.01_action3.game
target/release/leduc -m3 -n3 -p1 -b "2,4" -r 0.01 -s 3th_action -o games/leduc/leduc_m3n3p1b24r0.01_action3.game
target/release/leduc -m4 -n3 -p1 -b "2,4" -r 0.01 -s 3th_action -o games/leduc/leduc_m4n3p1b24r0.01_action3.game
target/release/leduc -m5 -n3 -p1 -b "2,4" -r 0.01 -s 3th_action -o games/leduc/leduc_m5n3p1b24r0.01_action3.game

target/release/leduc -m1 -n4 -p1 -b "2,4" -r 0.01 -s 3th_action -o games/leduc/leduc_m1n4p1b24r0.01_action3.game
target/release/leduc -m2 -n4 -p1 -b "2,4" -r 0.01 -s 3th_action -o games/leduc/leduc_m2n4p1b24r0.01_action3.game
target/release/leduc -m3 -n4 -p1 -b "2,4" -r 0.01 -s 3th_action -o games/leduc/leduc_m3n4p1b24r0.01_action3.game
target/release/leduc -m4 -n4 -p1 -b "2,4" -r 0.01 -s 3th_action -o games/leduc/leduc_m4n4p1b24r0.01_action3.game
target/release/leduc -m5 -n4 -p1 -b "2,4" -r 0.01 -s 3th_action -o games/leduc/leduc_m5n4p1b24r0.01_action3.game

target/release/leduc -m1 -n5 -p1 -b "2,4" -r 0.01 -s 3th_action -o games/leduc/leduc_m1n5p1b24r0.01_action3.game
target/release/leduc -m2 -n5 -p1 -b "2,4" -r 0.01 -s 3th_action -o games/leduc/leduc_m2n5p1b24r0.01_action3.game
target/release/leduc -m3 -n5 -p1 -b "2,4" -r 0.01 -s 3th_action -o games/leduc/leduc_m3n5p1b24r0.01_action3.game
target/release/leduc -m4 -n5 -p1 -b "2,4" -r 0.01 -s 3th_action -o games/leduc/leduc_m4n5p1b24r0.01_action3.game
target/release/leduc -m5 -n5 -p1 -b "2,4" -r 0.01 -s 3th_action -o games/leduc/leduc_m5n5p1b24r0.01_action3.game

target/release/leduc -m1 -n10 -p1 -b "2,4" -r 0.01 -s 3th_action -o games/leduc/leduc_m1n10p1b24r0.01_action3.game
target/release/leduc -m2 -n10 -p1 -b "2,4" -r 0.01 -s 3th_action -o games/leduc/leduc_m2n10p1b24r0.01_action3.game
target/release/leduc -m3 -n10 -p1 -b "2,4" -r 0.01 -s 3th_action -o games/leduc/leduc_m3n10p1b24r0.01_action3.game
target/release/leduc -m4 -n10 -p1 -b "2,4" -r 0.01 -s 3th_action -o games/leduc/leduc_m4n10p1b24r0.01_action3.game
target/release/leduc -m5 -n10 -p1 -b "2,4" -r 0.01 -s 3th_action -o games/leduc/leduc_m5n10p1b24r0.01_action3.game

target/release/leduc -m1 -n20 -p1 -b "2,4" -r 0.01 -s 3th_action -o games/leduc/leduc_m1n20p1b24r0.01_action3.game
target/release/leduc -m2 -n20 -p1 -b "2,4" -r 0.01 -s 3th_action -o games/leduc/leduc_m2n20p1b24r0.01_action3.game
target/release/leduc -m3 -n20 -p1 -b "2,4" -r 0.01 -s 3th_action -o games/leduc/leduc_m3n20p1b24r0.01_action3.game
target/release/leduc -m4 -n20 -p1 -b "2,4" -r 0.01 -s 3th_action -o games/leduc/leduc_m4n20p1b24r0.01_action3.game
target/release/leduc -m5 -n20 -p1 -b "2,4" -r 0.01 -s 3th_action -o games/leduc/leduc_m5n20p1b24r0.01_action3.game

echo "Generating standard games p=1 b=2,4 r=0.1 (general-sum)", without subgames

target/release/leduc -m1 -n2 -p1 -b "2,4" -r 0.01 -s none -o games/leduc/leduc_m1n2p1b24r0.01_nosub.game
target/release/leduc -m2 -n2 -p1 -b "2,4" -r 0.01 -s none -o games/leduc/leduc_m2n2p1b24r0.01_nosub.game
target/release/leduc -m3 -n2 -p1 -b "2,4" -r 0.01 -s none -o games/leduc/leduc_m3n2p1b24r0.01_nosub.game
target/release/leduc -m4 -n2 -p1 -b "2,4" -r 0.01 -s none -o games/leduc/leduc_m4n2p1b24r0.01_nosub.game
target/release/leduc -m5 -n2 -p1 -b "2,4" -r 0.01 -s none -o games/leduc/leduc_m5n2p1b24r0.01_nosub.game

target/release/leduc -m1 -n3 -p1 -b "2,4" -r 0.01 -s none -o games/leduc/leduc_m1n3p1b24r0.01_nosub.game
target/release/leduc -m2 -n3 -p1 -b "2,4" -r 0.01 -s none -o games/leduc/leduc_m2n3p1b24r0.01_nosub.game
target/release/leduc -m3 -n3 -p1 -b "2,4" -r 0.01 -s none -o games/leduc/leduc_m3n3p1b24r0.01_nosub.game
target/release/leduc -m4 -n3 -p1 -b "2,4" -r 0.01 -s none -o games/leduc/leduc_m4n3p1b24r0.01_nosub.game
target/release/leduc -m5 -n3 -p1 -b "2,4" -r 0.01 -s none -o games/leduc/leduc_m5n3p1b24r0.01_nosub.game

target/release/leduc -m1 -n4 -p1 -b "2,4" -r 0.01 -s none -o games/leduc/leduc_m1n4p1b24r0.01_nosub.game
target/release/leduc -m2 -n4 -p1 -b "2,4" -r 0.01 -s none -o games/leduc/leduc_m2n4p1b24r0.01_nosub.game
target/release/leduc -m3 -n4 -p1 -b "2,4" -r 0.01 -s none -o games/leduc/leduc_m3n4p1b24r0.01_nosub.game
target/release/leduc -m4 -n4 -p1 -b "2,4" -r 0.01 -s none -o games/leduc/leduc_m4n4p1b24r0.01_nosub.game
target/release/leduc -m5 -n4 -p1 -b "2,4" -r 0.01 -s none -o games/leduc/leduc_m5n4p1b24r0.01_nosub.game

target/release/leduc -m1 -n5 -p1 -b "2,4" -r 0.01 -s none -o games/leduc/leduc_m1n5p1b24r0.01_nosub.game
target/release/leduc -m2 -n5 -p1 -b "2,4" -r 0.01 -s none -o games/leduc/leduc_m2n5p1b24r0.01_nosub.game
target/release/leduc -m3 -n5 -p1 -b "2,4" -r 0.01 -s none -o games/leduc/leduc_m3n5p1b24r0.01_nosub.game
target/release/leduc -m4 -n5 -p1 -b "2,4" -r 0.01 -s none -o games/leduc/leduc_m4n5p1b24r0.01_nosub.game
target/release/leduc -m5 -n5 -p1 -b "2,4" -r 0.01 -s none -o games/leduc/leduc_m5n5p1b24r0.01_nosub.game

target/release/leduc -m1 -n10 -p1 -b "2,4" -r 0.01 -s none -o games/leduc/leduc_m1n10p1b24r0.01_nosub.game
target/release/leduc -m2 -n10 -p1 -b "2,4" -r 0.01 -s none -o games/leduc/leduc_m2n10p1b24r0.01_nosub.game
target/release/leduc -m3 -n10 -p1 -b "2,4" -r 0.01 -s none -o games/leduc/leduc_m3n10p1b24r0.01_nosub.game
target/release/leduc -m4 -n10 -p1 -b "2,4" -r 0.01 -s none -o games/leduc/leduc_m4n10p1b24r0.01_nosub.game
target/release/leduc -m5 -n10 -p1 -b "2,4" -r 0.01 -s none -o games/leduc/leduc_m5n10p1b24r0.01_nosub.game

target/release/leduc -m1 -n20 -p1 -b "2,4" -r 0.01 -s none -o games/leduc/leduc_m1n20p1b24r0.01_nosub.game
target/release/leduc -m2 -n20 -p1 -b "2,4" -r 0.01 -s none -o games/leduc/leduc_m2n20p1b24r0.01_nosub.game
target/release/leduc -m3 -n20 -p1 -b "2,4" -r 0.01 -s none -o games/leduc/leduc_m3n20p1b24r0.01_nosub.game
target/release/leduc -m4 -n20 -p1 -b "2,4" -r 0.01 -s none -o games/leduc/leduc_m4n20p1b24r0.01_nosub.game
target/release/leduc -m5 -n20 -p1 -b "2,4" -r 0.01 -s none -o games/leduc/leduc_m5n20p1b24r0.01_nosub.game

echo "Generating standard games p=1 b=2,4 r=0.0 (zero-sum)"

target/release/leduc -m1 -n2 -p1 -b "2,4" -r 0.0 -s none -o games/leduc/leduc_m1n2p1b24r0.0_nosub.game
target/release/leduc -m2 -n2 -p1 -b "2,4" -r 0.0 -s none -o games/leduc/leduc_m2n2p1b24r0.0_nosub.game
target/release/leduc -m3 -n2 -p1 -b "2,4" -r 0.0 -s none -o games/leduc/leduc_m3n2p1b24r0.0_nosub.game
target/release/leduc -m4 -n2 -p1 -b "2,4" -r 0.0 -s none -o games/leduc/leduc_m4n2p1b24r0.0_nosub.game
target/release/leduc -m5 -n2 -p1 -b "2,4" -r 0.0 -s none -o games/leduc/leduc_m5n2p1b24r0.0_nosub.game

target/release/leduc -m1 -n3 -p1 -b "2,4" -r 0.0 -s none -o games/leduc/leduc_m1n3p1b24r0.0_nosub.game
target/release/leduc -m2 -n3 -p1 -b "2,4" -r 0.0 -s none -o games/leduc/leduc_m2n3p1b24r0.0_nosub.game
target/release/leduc -m3 -n3 -p1 -b "2,4" -r 0.0 -s none -o games/leduc/leduc_m3n3p1b24r0.0_nosub.game
target/release/leduc -m4 -n3 -p1 -b "2,4" -r 0.0 -s none -o games/leduc/leduc_m4n3p1b24r0.0_nosub.game
target/release/leduc -m5 -n3 -p1 -b "2,4" -r 0.0 -s none -o games/leduc/leduc_m5n3p1b24r0.0_nosub.game

target/release/leduc -m1 -n4 -p1 -b "2,4" -r 0.0 -s none -o games/leduc/leduc_m1n4p1b24r0.0_nosub.game
target/release/leduc -m2 -n4 -p1 -b "2,4" -r 0.0 -s none -o games/leduc/leduc_m2n4p1b24r0.0_nosub.game
target/release/leduc -m3 -n4 -p1 -b "2,4" -r 0.0 -s none -o games/leduc/leduc_m3n4p1b24r0.0_nosub.game
target/release/leduc -m4 -n4 -p1 -b "2,4" -r 0.0 -s none -o games/leduc/leduc_m4n4p1b24r0.0_nosub.game
target/release/leduc -m5 -n4 -p1 -b "2,4" -r 0.0 -s none -o games/leduc/leduc_m5n4p1b24r0.0_nosub.game

target/release/leduc -m1 -n5 -p1 -b "2,4" -r 0.0 -s none -o games/leduc/leduc_m1n5p1b24r0.0_nosub.game
target/release/leduc -m2 -n5 -p1 -b "2,4" -r 0.0 -s none -o games/leduc/leduc_m2n5p1b24r0.0_nosub.game
target/release/leduc -m3 -n5 -p1 -b "2,4" -r 0.0 -s none -o games/leduc/leduc_m3n5p1b24r0.0_nosub.game
target/release/leduc -m4 -n5 -p1 -b "2,4" -r 0.0 -s none -o games/leduc/leduc_m4n5p1b24r0.0_nosub.game
target/release/leduc -m5 -n5 -p1 -b "2,4" -r 0.0 -s none -o games/leduc/leduc_m5n5p1b24r0.0_nosub.game

target/release/leduc -m1 -n10 -p1 -b "2,4" -r 0.0 -s none -o games/leduc/leduc_m1n10p1b24r0.0_nosub.game
target/release/leduc -m2 -n10 -p1 -b "2,4" -r 0.0 -s none -o games/leduc/leduc_m2n10p1b24r0.0_nosub.game
target/release/leduc -m3 -n10 -p1 -b "2,4" -r 0.0 -s none -o games/leduc/leduc_m3n10p1b24r0.0_nosub.game
target/release/leduc -m4 -n10 -p1 -b "2,4" -r 0.0 -s none -o games/leduc/leduc_m4n10p1b24r0.0_nosub.game
target/release/leduc -m5 -n10 -p1 -b "2,4" -r 0.0 -s none -o games/leduc/leduc_m5n10p1b24r0.0_nosub.game

target/release/leduc -m1 -n20 -p1 -b "2,4" -r 0.0 -s none -o games/leduc/leduc_m1n20p1b24r0.0_nosub.game
target/release/leduc -m2 -n20 -p1 -b "2,4" -r 0.0 -s none -o games/leduc/leduc_m2n20p1b24r0.0_nosub.game
target/release/leduc -m3 -n20 -p1 -b "2,4" -r 0.0 -s none -o games/leduc/leduc_m3n20p1b24r0.0_nosub.game
target/release/leduc -m4 -n20 -p1 -b "2,4" -r 0.0 -s none -o games/leduc/leduc_m4n20p1b24r0.0_nosub.game
target/release/leduc -m5 -n20 -p1 -b "2,4" -r 0.0 -s none -o games/leduc/leduc_m5n20p1b24r0.0_nosub.game

