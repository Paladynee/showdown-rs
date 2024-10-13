// const player_name = prompt("Enter your username:");

/** @type {HTMLCanvasElement} */
const canvas = document.getElementById("gameCanvas");
canvas.height = innerHeight;
canvas.width = innerWidth;
addEventListener("resize", () => {
    canvas.height = innerHeight;
    canvas.width = innerWidth;
});

/** @type {CanvasRenderingContext2D} */
const ctx = canvas.getContext("2d");

let bullets = [];

const my_player = {
    hp: 100,
    x: 0,
    y: 0,
};

const mousepos = { x: 0, y: 0 };

let other_players = [];

const move_speed = 500;

const pressed_keys = new Set();

addEventListener("keydown", (e) => {
    pressed_keys.add(e.key);
});

addEventListener("keyup", (e) => {
    pressed_keys.delete(e.key);
});

function handle_keys(dt) {
    if (pressed_keys.has("w")) {
        my_player.y -= 500 * dt;
    }
    if (pressed_keys.has("s")) {
        my_player.y += 500 * dt;
    }
    if (pressed_keys.has("a")) {
        my_player.x -= 500 * dt;
    }
    if (pressed_keys.has("d")) {
        my_player.x += 500 * dt;
    }
    if (pressed_keys.has("MouseLeft")) {
        fire_bullet(mousepos.x, mousepos.y);
    }
}

const addr = window.location.host;
const [ip, port] = addr.split(":");
const socket = new WebSocket("ws://" + ip + ":" + (parseInt(port, 10) + 1));

socket.onopen = function () {
    console.log("[WS] Connection to server established.");
};

socket.onmessage = function (event) {
    // Parse the game state update from the server
    const message = JSON.parse(event.data);
    // this is the sample message. it will not be used,
    // but it will be updated with what the server will return using this version of the software.
    // this is the skeleton. the specific values will be sent from the server.
    const sample_message = {
        recipient: "127.0.0.1:56000",
        game_state: {
            players: {
                "127.0.0.1:56000": {
                    x: 0,
                    y: 0,
                    hp: 100,
                },
            },
        },
    };

    // update other players
    let my_address = message.recipient;

    // while filtering out other players, add them to an array to replace the other_players array.
    let new_other_players = [];
    for (let [address, player] of Object.entries(message.game_state.players)) {
        if (address !== my_address) {
            new_other_players.push(player);
        }
    }

    other_players = new_other_players;
};

socket.onclose = function () {
    console.log("[CLIENT] Connection to server closed.");
};

let time = performance.now();
let last_ws_sent = performance.now();
let last_bullet_fired = performance.now();
loop();

addEventListener("mousedown", (e) => {
    // if it's a left click push it to pressed_keys
    if (e.button === 0) {
        pressed_keys.add("MouseLeft");
    }
});

addEventListener("mouseup", (e) => {
    // if it's a left click remove it from pressed_keys
    if (e.button === 0) {
        pressed_keys.delete("MouseLeft");
    }
});

addEventListener("mousemove", (e) => {
    mousepos.x = e.clientX;
    mousepos.y = e.clientY;
});

// function handle_click(event) {
//     let [x, y] = [event.clientX, event.clientY];
//     fire_bullet(x, y);
// }

function fire_bullet(posx, posy) {
    if (performance.now() - last_bullet_fired > 1000 / 5) {
        let angle = Math.atan2(posy - my_player.y, posx - my_player.x);
        let bullet = {
            x: my_player.x,
            y: my_player.y,
            velx: 1 * Math.cos(angle),
            vely: 1 * Math.sin(angle),
        };
        bullets.push(bullet);
        last_bullet_fired = performance.now();
    }
}

function loop() {
    let now = performance.now();
    let dt = (now - time) / 1000;
    time = now;

    ctx.clearRect(0, 0, canvas.width, canvas.height);

    for (let player of other_players) {
        ctx.fillStyle = "blue";
        ctx.fillRect(player.x - 25, player.y - 25, 50, 50);
        // draw the black hp bar background over other players
        ctx.fillStyle = "black";
        ctx.fillRect(player.x - 25 - 75, player.y - 25 - 75, 75 + 50 + 75, 25);
        // draw the hp percentage as green
        ctx.fillStyle = "green";
        ctx.fillRect(player.x - 25 - 75, player.y - 25 - 75, player.hp * 2, 25);
    }

    ctx.fillStyle = "red";
    ctx.fillRect(my_player.x - 25, my_player.y - 25, 50, 50);
    // draw the black hp bar background over my player
    ctx.fillStyle = "black";
    ctx.fillRect(my_player.x - 25 - 75, my_player.y - 25 - 75, 75 + 50 + 75, 25);
    // draw the hp percentage as green`
    ctx.fillStyle = "green";
    ctx.fillRect(my_player.x - 25 - 75, my_player.y - 25 - 75, my_player.hp * 2, 25);

    // draw bullets

    for (let bullet of bullets) {
        ctx.fillStyle = "black";
        ctx.fillRect(bullet.x - 5, bullet.y - 5, 10, 10);
    }

    handle_keys(dt);

    if (now - last_ws_sent > 1000 / 30) {
        if (socket.readyState !== WebSocket.OPEN) {
            console.log("Socket is not open, cannot send data");
        } else {
            send_game_data();
        }
        last_ws_sent = now;
    }

    requestAnimationFrame(loop);
}

function send_game_data() {
    const playerData = {
        hp: my_player.hp,
        x: my_player.x,
        y: my_player.y,
    };

    const game_state = {
        player: playerData,
        // new_bullets: bullets,
    };

    socket.send(JSON.stringify(game_state));
    // bullets = [];
}
