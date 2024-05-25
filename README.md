# DOCKER LOGS STREAMER VIA WEBSOCKET

Stream docker logs using Websocket to your browser.

# Build

```sh
cargo build --release
```

# Deploy

## Help Command:
```sh
cargo run -- --help

Usage: docker-container-log-streamer --stream-key <STREAM_KEY> --host <HOST> --port <PORT>

Options:
      --stream-key <STREAM_KEY>
          Stream Key used for Authentication

      --host <HOST>
          Host in IPV4 IP Address format

      --port <PORT>
          The port number to use

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

## Run using ```cargo```
```sh
cargo run -- --stream-key mystreamkey --host 0.0.0.0 --port 3090
```

## Run using the compiled binary file.

Build binary
```sh
cargo build --release
```

Execute binary
```sh
./target/release/docker-container-log-streamer --stream-key mystreamkey --host 0.0.0.0 --port 3090
```

# Testing

You use the random-messages scripts to build the docker container and execute the docker-compose to simular the docker logs streamer.

1. Build the random-messages container.
```sh
cd tests
docker build . -t random_messages:latest
```

2. Execute docker-compose.
```sh
docker-compose -f docker-compose-random-messages.yml up
```

You can simulate the log streaming using the provided frontend app.

1. Execute docker-compose for the frontend.
```sh
cd tests
docker-compose -f docker-compose-frontend.yaml up -d
```

2. Look for the container of the frontend.
```sh
docker ps 

Example:
CONTAINER ID   IMAGE                    COMMAND                  CREATED       STATUS             PORTS                  NAMES
e6b8bedb2193   node                     "docker-entrypoint.s…"   3 hours ago   Up 3 hours         0.0.0.0:7050->80/tcp   tests-nginx-1
```

3. Grab the container id
```sh
docker exec -it e6b8bedb2193 "bash"
```

4. Run the development server.
```sh
cd app
yarn dev
```

5. Open browser then go to
```
http://localhost:7090
```

# Developer
JP Mateo (jpmateo022@gmail.com)