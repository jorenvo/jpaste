jpaste
======

This is a pastebin-like service meant to be used with cURL. It uses Redis for storage and pastes expire after one month.

Example:
  $ echo hi | curl -F 'j=<-' https://jvo.sh/j
  https://jvo.sh/ZnD9BBwj
  $ curl https://jvo.sh/ZnD9BBwj
  hi

Easiest way to build and run is with cargo:
  $ cargo run

Some configuration is possible with environment variables (default values are shown):
  - JPASTE_REDIS='redis://127.0.0.1/'
  - JPASTE_PUBLIC_URL='http://127.0.0.1'
  - JPASTE_LOCALHOST_PORT='3030'