#!/bin/bash

# timezone
TZ='America/Sao_Paulo'
export TZ

echo "Uploading World backup to S3..."

# backup world
zip -r worlds.zip /home/steam/.config/unity3d/IronGate/Valheim/worlds/
aws s3api put-object --body "worlds.zip" --bucket "amnesicbit" --key "valheim/backups/worlds-$(date +%d_%m_%y-%H_%M).zip" --acl "public-read" --content-type "application/zip"
rm worlds.zip

echo "Starting Valheim server..."

# start server
runuser -l steam -c 'rm -f /home/steam/valheim/nohup.out'
runuser -l steam -c 'cd /home/steam/Steam; ./steamcmd.sh +login anonymous +force_install_dir /home/steam/valheim +app_update 896660 +quit'
runuser -l steam -c 'cd /home/steam/valheim; nohup ./start_server.sh &'
