#!/bin/bash

useradd -m steam
#sudo apt-get install lib32gcc1 libsdl2-2.0-0:i386 tmux screen -y
yum install glibc.i686 libstdc++.i686 tmux screen libsdl2-2.0-0:i386 -y
#su - steam
sudo -iu steam
mkdir ~/Steam && cd ~/Steam
curl -sqL "https://steamcdn-a.akamaihd.net/client/installer/steamcmd_linux.tar.gz" | tar zxvf -
cd ~/Steam
./steamcmd.sh +login anonymous +force_install_dir /home/steam/valheim +app_update 896660 +quit