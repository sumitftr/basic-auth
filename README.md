# Features

- Consistency across sessions: When a user logs in with multiple devices having different sessions, the user data will stay consistent across all devices upon reload.
- User sent cookies are not directly stored in database. User sent cookies are signed with the `SECRET_KEY` and the unsigned version is stored in database.
- Auto Refreshing Sessions: If a user tries to log in within 7 days after the session has expired then the user is automatically logged back in.

# Limitations & Use Cases

- This server is dependent on `moka` (a fast, concurrent cache library) instead of other key value databases like redis, memcached, valkey, dragonfly, etc. So using load balancers without session affinity (sticky sessions) will break the origin servers
- The session affinity ttl (Time to Live) must be equal to `common::session::Session::MEM_CACHE_DURATION` for consistency

## Project Setup

> Create a `.env` file inside project root
> And set the following environment variables inside `.env` file

```dotenv
SOCKET=your_ip:your_port
SECRET_KEY=your_secret_key_for_signing_cookies
SERVICE_NAME=your_service_name
SERVICE_DOMAIN=your_service_domain_with_scheme

# Database
DATABASE_URL=your_postgres_database_url

# Object Storage
BUCKET_ACCESS_KEY=your_bucket_access_key
BUCKET_SECRET_KEY=your_bucket_secret_key
BUCKET_ID=your_bucket_id
BUCKET_ENDPOINT=your_bucket_endpoint
BUCKET_NAME=your_bucket_name
BUCKET_REGION=your_bucket_region
BUCKET_PUBLIC_URL=your_bucket_public_url

# Email
SMTP_KEY=your_smtp_key
SMTP_HOST=your_smtp_host
NOREPLY_EMAIL=your_noreply_email

# OAuth
GOOGLE_CLIENT_ID=your_google_client_id
GOOGLE_CLIENT_SECRET=your_google_client_secret
```

## Run your project
```
  cargo run --release
```
> For hot reloading use:
```
  cargo watch -x run
```
