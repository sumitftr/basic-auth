## Project Setup

> Create a `.env` file inside project root
> And set the following environment variables inside `.env` file

> Set your **`SERVER SOCKET IP`** and **`PORT`**
```
  SOCKET=<socket_ip>:<port>
```

> Set your **`DATABASE URI`**
```
  DATABASE_URI=<your_database_uri>
```

> Set your **`DATABASE NAME`**
```
  DATABASE_NAME=<your_database_name>
```

> Set your **`SECRET KEY`**
```
  SECRET_KEY=<your_secret_key>
```

## Run your project
```
  cargo run --release
```
> For hot reloading use:
```
  cargo watch -x run
```
