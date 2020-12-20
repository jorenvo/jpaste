jpaste
======

This is a pastebin-like service meant to be used with cURL. It uses Redis for storage and pastes expire after one month.

Example:
  $ echo hi | curl -F 'j=<-' https://jvo.sh/j
  https://jvo.sh/ZnD9BBwj
  $ curl https://jvo.sh/ZnD9BBwj
  hi

Use cargo to build and run:
  $ cargo run --release

Todo:
- env variable to configure URL in help