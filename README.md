# Echo Serv

This `echo-serv` command accepts HTTP requests and returns the request information.

## TODO

**This is still the first version.**

## Usage from Docker Hub

1. `docker run --rm -p 7878:7878 arkbig/echo-serv`
2. `curl http://localhost:7878/echo`
   - The response is a json of request information.
3. Access <http://localhost:7878> using browser to display the request history.

## Usage from GitHub

1. Build and up container

    ```sh
    git clone https://github.com/arkbig/echo-serv.git
    cd echo-serv
    deploy/docker/replace-env.sh
    docker compose -f deploy/docker/compose.yaml up -d
    ```

2. `curl http://localhost:7878/echo`
   - The response is a json of request information.
3. Access <http://localhost:7878> using browser to display the request history.
