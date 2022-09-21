#!/bin/bash
#This script builds the web assembly targets and copies them to /web/wasm
#wasm-pack is required for the build process
cd ./wasm
(
    exec wasm-pack build --target web
    if [ &? -ne 0]
    then
        exit
    fi
)
cd ..
if ! [[ -f "web/public/wasm" ]]
then 
    mkdir web/public/wasm
fi
yes | cp -rf ./wasm/pkg/acquire_rs_wasm.js ./web/public/wasm
yes | cp -rf ./wasm/pkg/acquire_rs_wasm_bg.wasm ./web/public/wasm/
echo "wasm files have been generated and copied to /web/wasm"