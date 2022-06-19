#!/bin/bash
#cargo run --release -- --render funky_beat.ogg ms7.mid
#cargo run --release -- --render cybergrind.ogg cybergrind.mid
#cargo run --release -- --render 2reality.ogg 2reality.mid
#cargo run --release -- --render lobby.ogg lobby.mid
#cargo run --release -- --render halo.ogg halo.mid
#cargo run --release -- --render pod.ogg pod.mid
#cargo run --release -- --render dragonage.ogg dragonage.mid

cargo run --release -- --render ms7.ogg ms72.mid

#rsync -Pvr resources/ ../phantoma/resources/
