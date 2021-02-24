#!/bin/bash

aws ecr get-login-password --region us-east-1 | docker login --username AWS --password-stdin 243101742269.dkr.ecr.us-east-1.amazonaws.com
docker build -t valheim .
docker tag valheim:latest 243101742269.dkr.ecr.us-east-1.amazonaws.com/valheim:latest
docker push 243101742269.dkr.ecr.us-east-1.amazonaws.com/valheim:latest