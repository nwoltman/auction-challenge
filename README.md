# auction_challenge

Solution uses Rust 1.37.

Run with Docker:

```sh
# sudo if necessary
docker build -t challenge .
docker run -i -v /path/to/config.json:/auction/config.json challenge < /path/to/input.json
```

Output is a JSON array where each item is an array of winning bids for the corresponding input auction.
