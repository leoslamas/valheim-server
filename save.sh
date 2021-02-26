#!/bin/bash

zip -r worlds.zip /home/steam/.config/unity3d/IronGate/Valheim/worlds/
aws s3api put-object --body "worlds.zip" --bucket "amnesicbit" --key "valheim/backups/worlds-$(date +%d_%m_%y-%H_%M).zip" --acl "public-read" --content-type "application/zip"
rm worlds.zip