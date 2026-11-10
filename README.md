# Local LLM cloud for my personal use

Another project I wrote in Rust that has no need to be in rust. Monorepo for the client/server architecture for a local llm "cloud" so I can milk a little more use out of my GPU since apparently the only thing that matters in this current world is manufacturing silicon for hyperscaler data centers and us consumers can go walk into the ocean.

Run the client on my laptop, run the server on my desktop, use the client to control the server from the local network to serve LLMs through [llama.cpp](https://llama.app/). Runs in a (very poorly built) tui using the excellent [Ratatui](https://ratatui.rs/) crate.

Completely unnecessary, but I get to learn how to do client <-> server communication via websockets in Rust and learn how to write an immediate mode application with async channels.
