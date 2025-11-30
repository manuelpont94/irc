#!/usr/bin/env python3
import socket
import threading

HOST = "0.0.0.0"
PORT = 6667


class ClientState:
    def __init__(self):
        self.nick = None
        self.user = None
        self.registered = False


def send_line(conn, line):
    print(list((line + "\r\n").encode("utf-8")))
    conn.send((line + "\r\n").encode("utf-8"))

    print("SEND:", line)


def handle_client(conn, addr):
    print(f"Client connected: {addr}")
    state = ClientState()

    while True:
        data = conn.recv(4096)
        if not data:
            break

        for line in data.decode("utf-8", errors="ignore").split("\r\n"):
            if not line.strip():
                continue

            print("RECV:", line)
            parts = line.split()
            cmd = parts[0].upper()

            # ---------- NICK ----------
            if cmd == "NICK":
                state.nick = parts[1]
                try_register(conn, state)

            # ---------- USER ----------
            elif cmd == "USER":
                # USER <username> <mode> <unused> :<realname>
                state.user = parts[1]
                try_register(conn, state)

            # ---------- PONG ----------
            elif cmd == "PONG":
                continue  # ignore

            # ---------- JOIN ----------
            elif cmd == "JOIN" and state.registered:
                channel = parts[1]
                send_line(conn, f":{state.nick}!{state.user}@localhost JOIN {channel}")
                send_line(conn, f":server 331 {state.nick} {channel} :No topic set")
                send_line(conn, f":server 353 {state.nick} = {channel} :{state.nick}")
                send_line(
                    conn, f":server 366 {state.nick} {channel} :End of /NAMES list."
                )

            # ---------- PING ----------
            elif cmd == "PING":
                send_line(conn, f"PONG {parts[1]}")

            else:
                send_line(
                    conn, f":server 421 {state.nick or '*'} {cmd} :Unknown command"
                )

    conn.close()
    print(f"Client disconnected: {addr}")


def try_register(conn, state):
    """Send welcome numerics once both NICK and USER are received."""
    if state.registered:
        return
    if not state.nick or not state.user:
        return

    state.registered = True

    send_line(
        conn,
        f":server 001 {state.nick} :Welcome to the IRC Network {state.nick}!{state.user}@localhost",
    )
    send_line(
        conn, f":server 002 {state.nick} :Your host is server, running minimal-irc"
    )
    send_line(conn, f":server 003 {state.nick} :This server was created today")
    send_line(conn, f":server 004 {state.nick} server v0.1 o o")
    send_line(conn, "PING :12345")  # optional


def main():
    print(f"IRC server running on {HOST}:{PORT}")
    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    s.bind((HOST, PORT))
    s.listen(5)

    while True:
        conn, addr = s.accept()
        threading.Thread(target=handle_client, args=(conn, addr), daemon=True).start()


if __name__ == "__main__":
    main()
