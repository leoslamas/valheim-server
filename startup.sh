#!/bin/bash

# timezone
TZ='America/Sao_Paulo'
export TZ

echo "Uploading backup to S3..."

# backup world
zip -r worlds.zip /home/steam/.config/unity3d/IronGate/Valheim/worlds/
aws s3api put-object --body "worlds.zip" --bucket "amnesicbucket" --key "valheim/backups/worlds-$(date +%d_%m_%y-%H_%M).zip" --acl "public-read" --content-type "application/zip"
rm worlds.zip

# start server
sudo -iu steam
cd /home/steam/valheim
rm nohup.out
nohup ./start_server.sh &
