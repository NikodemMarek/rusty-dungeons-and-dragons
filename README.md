# Rusty Dungeons And Dragons

A text-based rpg, with GPT powered story. This is inspired by DND, but not strictly DND.

## Environment Variables

To run this project, you will need to have a OpenAI api token in your environment variables

You can add it to your .env, with

```sh
echo "OPENAI_API_KEY={your_key}" > .env
```

## Run Locally

Clone the project

```sh
git clone https://github.com/NikodemMarek/rusty-dungeons-and-dragons.git
cd rusty-dungeons-and-dragons
```

Run the project with cargo

```sh
cargo run
```

For ease of development, use [cargo-watch](https://crates.io/crates/cargo-watch)

```sh
cargo-watch -x run
```

If you use NixOS, use

```sh
nix develop
d
```
