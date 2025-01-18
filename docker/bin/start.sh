#!/bin/bash
SHELL_FOLDER=$(cd "$(dirname "$0")";pwd)

cd bin && LD_LIBRARY_PATH=$(pwd) ./tts_server -c ../config/tts.yaml

#while true; do sleep 10; done
