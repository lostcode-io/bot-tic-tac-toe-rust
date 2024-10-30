#!/bin/bash

if [[ -z "$SECRET" ]]; then echo "SECRET is not set"; exit 1; fi

echo "Starting server on port $PORT"

/app/bot -p 8080 -s ${SECRET}

