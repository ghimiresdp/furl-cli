# Basic Auth test server

A local nginx server protected by HTTP Basic Authentication, for testing
`furl -u` against real files of different sizes. Unlike public services
such as httpbin.org, nginx supports `Range` requests, so furl downloads the
larger files here in parallel chunks, with every chunk request authenticated.

The automated tests in `crates/furl-core/tests/basic_auth.rs` (part of
`cargo test`) don't need this; it is for manual and end-to-end testing.

## Usage

From the repository root:

```bash
./e2e-basic-auth.sh       # build furl, start the server, download and verify everything
docker compose down -v    # stop the server and delete the generated files
```

`./e2e-basic-auth.sh` is an end-to-end check of the real `furl` binary, not
the project's test suite (that is `cargo test --workspace`). It builds furl
from the workspace, and starts the server with `docker compose up -d` if it
isn't running yet (the first start generates the test files, which takes a
few seconds). It then downloads every file with
credentials and checks it against `SHA256SUMS`. It also checks that
credentials in the URL work, and that a wrong or missing password fails with
`401 Unauthorized` without leaving a file behind. To test a different binary,
pass its path: `./e2e-basic-auth.sh path/to/furl`.

To use the server for manual testing only, run `docker compose up -d`.

The server listens on port 8080. To use another port, set `FURL_TEST_PORT`,
e.g. `FURL_TEST_PORT=9000 ./e2e-basic-auth.sh`. If the server is already
running, set it the same way for `docker compose` too.

The compose file is [docker-compose.yml](../../docker-compose.yml) at the
repository root; this directory holds the nginx config and the `htpasswd`
file it mounts.

## Credentials and files

- Username `furl`, password `furl-test`. These are test-only credentials; to
  change them, regenerate `htpasswd` with
  `printf "furl:%s\n" "$(openssl passwd -apr1 'new-password')" > htpasswd`.
- `http://localhost:8080/<file>` requires authentication, and
  `http://localhost:8080/public/<file>` serves the same files without it.

| File         | Size       | What it exercises                               |
| ------------ | ---------- | ----------------------------------------------- |
| `empty.bin`  | 0 B        | empty file (server answers the probe with 416)  |
| `tiny.bin`   | 1 B        | single byte                                     |
| `small.bin`  | 300 KB     | under 1 MB, downloaded without threads          |
| `text.txt`   | ~1.3 MB    | text content                                    |
| `medium.bin` | 5 MB       | a few chunks                                    |
| `large.bin`  | 64 MB      | many chunks                                     |
| `huge.bin`   | 512 MB     | long-running download with many chunks          |
| `SHA256SUMS` | —          | checksums of all the files above                |

Manual examples:

```bash
furl http://localhost:8080/large.bin -u furl:furl-test -o /tmp -t 16
furl http://localhost:8080/large.bin -u furl            # prompts for password
furl http://localhost:8080/large.bin -u furl:wrong      # fails with 401
```
