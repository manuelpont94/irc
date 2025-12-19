This project implements a basic IRC (Internet Relay Chat) server and client in Rust, designed as a learning exercise to explore network programming, asynchronous I/O with Tokio, and the IRC protocol. The server supports essential commands for user registration, channel management, and messaging, allowing multiple clients to connect and communicate in real-time. A simple Python-based IRC server is also included for comparison and reference.

## Limitations
- **Single-host deployment**: The server binds only to `127.0.0.1:6667`, restricting connections to the local machine. It does not support remote access, multi-server federation, or distributed networks.
- **Partial IRC protocol compliance**: Only a subset of IRC commands is implemented (e.g., NICK, USER, JOIN, PART, PRIVMSG, PONG, QUIT, TOPIC, NAMES). Advanced features like user modes, channel operators, server-to-server links, or extensions (e.g., SSL/TLS encryption) are not supported.
- **No persistence**: User data, channel history, and state are not saved across server restarts.
- **Basic error handling**: While errors are logged, the implementation lacks robust recovery mechanisms for network failures or malformed inputs.
- **Client implementation**: The Rust client is a placeholder and not functional; users must rely on external IRC clients to connect to the server.
- **Performance and scalability**: Designed for small-scale, local testing; it may not handle high concurrency or large numbers of users efficiently due to in-memory state management.

Work in progress...
