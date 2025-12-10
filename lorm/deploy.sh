#!/usr/bin/env sh

APIFY_VERSION=$(cargo metadata --manifest-path apps/apify/Cargo.toml --format-version 1 | jq -r '.[] | map(select(.name == "apify")) | .[0].version' 2>/dev/null)
docker build . -t weavly/apify:APIFY_VERSION

#docker run -it --name apify-adstxt -e RUST_LOG=info -e WEB_PORT=3001 -p 3001:3001 weavly/apify run ads-txt
