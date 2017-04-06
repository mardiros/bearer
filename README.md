# Bearer

[![Build Status](https://travis-ci.org/mardiros/bearer.svg?branch=master)](https://travis-ci.org/mardiros/bearer)
[![Current Crates.io Version](https://img.shields.io/crates/v/bearer.svg)](https://crates.io/crates/bearer)
[![Latests Documentation](https://docs.rs/bearer/badge.svg)](https://docs.rs/crate/bearer)

## Basics

`bearer` is a command line utility to generate Authorization HTTP header
for bearer tokens. See [RFC 6750](https://tools.ietf.org/html/rfc6750).

`bearer` comes with a `--register` that will ask you the OAuth 2.0
client information to initialize the access and refresh tokens.

Afterwhat, it generate a header that can be used in a curl command:

```

    $ curl -H "$(bearer <client_name>)" "https://<oauth2 api>" | jq

```

Clients that received refresh token will automatically consume them to retrieve
new access token before they expires.

Otherwise the command `--refresh` has to be used to get a new access token.

## Installation

Currently, `bearer` is installable using `cargo`

```

    $ cargo install bearer

```

Note:

    Cargo install binaries in `$HOME/.cargo/bin` directory. Make sure it is in
    your `$PATH` environment.


## Register a client

```

    $ bearer my-client-name --register

```

You have to follow the instruction of the command.

```
    Before continue, register the a client with the following url to the OAuth2 Provider:

    http://localhost:6750/callback

    Ensure your port is not already open by another service.
    If the provider require a https url, please run an https reverse proxy before continue.

    Enter the OAuth2.0 Provider Name:
    Enter the Client Id: 
    Enter the Client Secret: 
    Enter the scope (optional): profile email

    Visit to finish the configuration: http://localhost:6750/callback

```

After input thoses informations your have to open your browser and visit
the `http://localhost:6750/callback` url to retrieve tokens. Then
the message below confirm everything is ok.

```
    Tokens retrieved succesfully
```

## List registered client

```

    $ bearer --list
    my-client-name

```

## Generating Authorizaton header


```

    $ bearer my-client-name
    Authorization: Bearer GlwlBMvJI

```

## Refreshing token

**This is useless if your OAuth2 provider send a refresh token.**

Otherwise,

if the access token has been retrieved without refresh token,
it cannot be replaced automatically by a new one. Command will
failed when the token expires.


```

    $ bearer -c config my-client-without-refresh-token
    ERROR: Client must be refreshed. (No Refresh Token)
    $ bearer my-client-without-refresh-token --refresh

    Visit to finish the configuration: http://localhost:6750/callback

```

## Supported Platform

`bearer` has been developped under Linux.

It may not work under other operating system. (Not tests)
