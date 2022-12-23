# ofcr.se

This repository contains the source code for [my personal website](https://ofcr.se/).

## Site

The site is built using [Astro](https://astro.build/).

## Server

The website is served by a Rust application (mainly meant for me to learn Rust).

Environment variables:
- `PORT`: port to listen on (default: 8080)
- `SITE_URL`: full URL of the site (default: `http://localhost:8080/`)
- `SHORTLINKS_FILE`: JSON file containing shortlinks for redirecting
- `GOATCOUNTER_URL`: URL to the GoatCounter instance used for proxying
