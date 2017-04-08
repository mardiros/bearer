# CHANGELOG

## bearer 0.2.2 2017-04-08

 * Update documentation 

## bearer 0.2.1 2017-04-05

 * Add a `bearer --list` command.
 * Add a `bearer [client] --refresh` command.

## bearer 0.2.0 2017-04-02

 * Simplify registrations on known providers.
 * Break the .toml file format. Store the client config in a `[client]`. Store
   `token_url` and `authorize_url` that replace the `server_url`. Also store
   the provider name.
 * Add unit tests.

## bearer 0.1.0 2017-03-22

 * Initial release that initialize token with authorization and update then with refresh token.
