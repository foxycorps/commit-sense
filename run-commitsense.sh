#!/bin/bash

# Check if API key is provided
if [ -z "$1" ]; then
  echo "Error: OpenAI API key is required"
  echo "Usage: ./run-commitsense.sh YOUR_API_KEY [additional arguments]"
  exit 1
fi

API_KEY="$1"
shift  # Remove the API key from the arguments

# Run the Docker container with the API key as an environment variable
docker run -e OPENAI_API_KEY="$API_KEY" -t commitsense "$@"
