// const player_name = prompt("Enter your username:");

/** @type {HTMLCanvasElement} */
const canvas = document.getElementById("gameCanvas");
canvas.height = innerHeight;
canvas.width = innerWidth;

/** @type {CanvasRenderingContext2D} */
const ctx = canvas.getContext("2d");

const player_size = 50;
const bullet_size = 10;
const hp_bar_width = 110;
const hp_bar_height = 11;
const hp_bar_offset_y = 55;
const hp_bar_outline_width = 3;

const mousepos = { x: 0, y: 0 };
const move_speed = 200;
const bullet_speed = 500;
let fire_rate = 5;
const pressed_keys = new Set();
let my_address = undefined;

const game_state = {
    player: {
        x: innerWidth / 2,
        y: innerHeight / 2,
        hp: 100,
    },
    all_players: {},
    unfired_bullets: [],
    bullets: [],
};

addEventListener("keydown", (e) => {
    pressed_keys.add(e.key);
});

addEventListener("keyup", (e) => {
    pressed_keys.delete(e.key);
});

addEventListener("resize", () => {
    canvas.height = innerHeight;
    canvas.width = innerWidth;
});

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

addEventListener("contextmenu", (e) => {
    e.preventDefault();
});

const socket = get_socket(handle_server_message);

let time = performance.now();
let last_ws_sent = performance.now();
let last_bullet_fired = performance.now();

loop();

setInterval(() => {
    let now = performance.now();
    if (now - last_ws_sent > 1000 / 120) {
        if (socket.readyState !== WebSocket.OPEN) {
            console.log("Socket is not open, cannot send data");
        } else {
            send_game_data();
        }
        last_ws_sent = now;
    }
}, 1000 / 100);

// function handle_click(event) {
//     let [x, y] = [event.clientX, event.clientY];
//     fire_bullet(x, y);
// }

function fire_bullet(posx, posy) {
    if (performance.now() - last_bullet_fired > 1000 / fire_rate) {
        let angle = Math.atan2(posy - game_state.player.y, posx - game_state.player.x);
        let bullet = {
            x: game_state.player.x,
            y: game_state.player.y,
            velx: bullet_speed * Math.cos(angle),
            vely: bullet_speed * Math.sin(angle),
            life: 1,
            owner: my_address,
        };
        game_state.unfired_bullets.push(bullet);
        last_bullet_fired = performance.now();
    }
}

function loop() {
    let now = performance.now();
    let dt = (now - time) / 1000;
    time = now;

    ctx.clearRect(0, 0, canvas.width, canvas.height);

    for (let player_ip in game_state.all_players) {
        if (player_ip === my_address) {
            continue;
        }

        let player = game_state.all_players[player_ip];

        // draw the player ring
        ctx.strokeStyle = "blue";
        ctx.lineWidth = 7;
        ctx.beginPath();
        ctx.arc(player.x, player.y, player_size / 2, 0, Math.PI * 2);
        ctx.stroke();

        // draw the health bar
        // health bar background outline
        ctx.fillStyle = "black";
        ctx.fillRect(
            player.x - hp_bar_width / 2 - hp_bar_outline_width,
            player.y - hp_bar_offset_y - hp_bar_height / 2 - hp_bar_outline_width,
            hp_bar_width + hp_bar_outline_width * 2,
            hp_bar_height + hp_bar_outline_width * 2
        );

        // health bar background
        ctx.fillStyle = "#333";
        ctx.fillRect(player.x - hp_bar_width / 2, player.y - hp_bar_offset_y - hp_bar_height / 2, hp_bar_width, hp_bar_height);

        // health bar partially filled
        ctx.fillStyle = "#00ff00";
        ctx.fillRect(player.x - hp_bar_width / 2, player.y - hp_bar_offset_y - hp_bar_height / 2, (player.hp / 100) * hp_bar_width, hp_bar_height);
    }

    if (my_address !== undefined) {
        // draw my player
        let my_player = game_state.all_players[my_address];

        // draw the player ring
        ctx.strokeStyle = "aqua";
        ctx.lineWidth = 7;
        ctx.beginPath();
        ctx.arc(my_player.x, my_player.y, player_size / 2, 0, Math.PI * 2);
        ctx.stroke();

        // draw the health bar
        // health bar background outline
        ctx.fillStyle = "black";
        ctx.fillRect(
            my_player.x - hp_bar_width / 2 - hp_bar_outline_width,
            my_player.y - hp_bar_offset_y - hp_bar_height / 2 - hp_bar_outline_width,
            hp_bar_width + hp_bar_outline_width * 2,
            hp_bar_height + hp_bar_outline_width * 2
        );

        // health bar background
        ctx.fillStyle = "#333";
        ctx.fillRect(my_player.x - hp_bar_width / 2, my_player.y - hp_bar_offset_y - hp_bar_height / 2, hp_bar_width, hp_bar_height);

        // health bar partially filled
        ctx.fillStyle = "#00ff00";
        ctx.fillRect(my_player.x - hp_bar_width / 2, my_player.y - hp_bar_offset_y - hp_bar_height / 2, (my_player.hp / 100) * hp_bar_width, hp_bar_height);
    }

    // draw bullets
    for (let bullet of game_state.bullets) {
        ctx.fillStyle = "black";
        ctx.beginPath();
        ctx.arc(bullet.x, bullet.y, bullet_size / 2, 0, Math.PI * 2);
        ctx.fill();
    }

    handle_keys(dt);

    requestAnimationFrame(loop);
}

function send_game_data() {
    const playerData = {
        x: game_state.player.x,
        y: game_state.player.y,
    };

    const packet = {
        player: playerData,
        new_bullets: game_state.unfired_bullets,
    };

    socket.send(JSON.stringify(packet));
    game_state.unfired_bullets.length = 0;
}

function handle_keys(dt) {
    if (pressed_keys.has("w")) {
        game_state.player.y -= move_speed * dt;
    }
    if (pressed_keys.has("s")) {
        game_state.player.y += move_speed * dt;
    }
    if (pressed_keys.has("a")) {
        game_state.player.x -= move_speed * dt;
    }
    if (pressed_keys.has("d")) {
        game_state.player.x += move_speed * dt;
    }
    if (pressed_keys.has("MouseLeft")) {
        fire_bullet(mousepos.x, mousepos.y);
    }
}

function get_socket(fn) {
    const addr = window.location.host;
    const [ip, port] = addr.split(":");
    const socket = new WebSocket("ws://" + ip + ":" + (parseInt(port, 10) + 1));

    socket.onopen = function () {
        console.log("[WS] Connection to server established.");
    };

    socket.onmessage = fn;

    socket.onclose = function () {
        console.log("[CLIENT] Connection to server closed.");
    };

    return socket;
}

function handle_server_message(event) {
    const message = JSON.parse(event.data);
    my_address = message.recipient;

    game_state.all_players = message.game_state.players;
    game_state.bullets = message.game_state.bullets;

    // find the distance between client myplayer and server myplayer, and if it surpasses a certain threshold, update the client myplayer
    let server_my_player = game_state.all_players[my_address];
    let client_my_player = game_state.player;
    let distance = Math.sqrt((server_my_player.x - client_my_player.x) ** 2 + (server_my_player.y - client_my_player.y) ** 2);
    if (distance > 50) {
        game_state.player = server_my_player;
    }
}
