# kybe-backend

## Setup

Download the docker-compose.yml

``curl https://raw.githubusercontent.com/2kybe3/kybe-backend/refs/heads/main/docker-compose.yml``

Start the application

``docker compose up -d``

Edit the configuration

``vim backend/config.toml``

Restart the application

``docker compose up -d --force-recreate``


## Future Plans

1. have a kybe-cli application allowing users to use the backend
2. have a user system via the cli to register / login / change_pass / change_mail / verify_mail etc.
3. allow users to shorten links (also allow limited views, password protection (prob simple http auth))
