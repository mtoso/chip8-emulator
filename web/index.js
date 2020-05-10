import init, { Cartridge, Cpu } from './chip8.js'

const CANVAS_WIDTH = 64;
const CANVAS_HEIGHT = 32;
const GAME_SPEEDS = [
    1,
    2,
    3,
];

const ROMS = [
    'IBM',
    'INVADERS',
    'PONG2',
    'TETRIS',
    'TIMEBOMB',
    'UFO',
    'WIPEOFF',
];

const romsSelect = document.getElementById("roms");
const runButton = document.getElementById("run");
const gameSpeeds = document.getElementById("game_speeds");

ROMS.forEach(rom => {
    const opt = document.createElement('option');
    opt.appendChild(document.createTextNode(rom));
    opt.value = rom;
    romsSelect.appendChild(opt);
});

GAME_SPEEDS.forEach(speed => {
    const opt = document.createElement('option');
    opt.appendChild(document.createTextNode(`${speed}X`));
    opt.value = speed;
    gameSpeeds.appendChild(opt);
});

function initCanvas(width, height) {
    const canvas = document.getElementById("canvas");
    const ctx = canvas.getContext("2d");
    ctx.fillStyle = "balck";
    ctx.fillRect = (0, 0, width, height);
    return ctx;
}

function updateCanvas(displayState, ctx, width, height) {
    const imageData = ctx.createImageData(width, height);
    for (let i = 0; i < displayState.length; i++) {
        imageData.data[i * 4] = displayState[i] === 1 ? 0x33 : 0;
        imageData.data[i * 4 + 1] = displayState[i] === 1 ? 0xff : 0;
        imageData.data[i * 4 + 2] = displayState[i] === 1 ? 0x66 : 0;
        imageData.data[i * 4 + 3] = 255;
    }
    ctx.putImageData(imageData, 0, 0);
}

async function loadRom(rom, emulator) {
    const response = await window.fetch(`roms/${rom}.ch8`);
    const program = await response.arrayBuffer();
    const cartridge = Cartridge.new(new Uint8Array(program));
    emulator.reset();
    emulator.load_cartridge(cartridge);
}

const mainCtx = initCanvas(CANVAS_WIDTH, CANVAS_HEIGHT);
(async function run() {
    await init();

    const emulator = Cpu.new();

    romsSelect.value = 'WIPEOFF';
    await loadRom('WIPEOFF', emulator);

    let gameSpeed = gameSpeeds.value = 1;
     
    let running = false;
    const runloop = () => {
        if (running) {
            let result;
            // batch instructions
            for (let i = 0; i < (gameSpeed * 10); i++) {
                result = emulator.execute_cycle();
            }
            const displayState = result.get_display_state();
            updateCanvas(displayState, mainCtx, CANVAS_WIDTH, CANVAS_HEIGHT);
        }
        window.requestAnimationFrame(runloop);
    }
    window.requestAnimationFrame(runloop);

    runButton.addEventListener("click", () => {
        if (running) {
            running = false;
            runButton.innerHTML = "Start";
        } else {
            running = true;
            runButton.innerHTML = "Stop";
        }
    });

    romsSelect.addEventListener("change", async(e) => {
        await loadRom(e.target.value, emulator);
    });

    gameSpeeds.addEventListener("change", async(e) => {
        gameSpeed = e.target.value;
    });

    document.addEventListener('keydown', event => {
        const key = event.key;
        emulator.keypad_down(key);
    });

    document.addEventListener('keyup', event => {
        const key = event.key;
        emulator.keypad_up(key);
    });

})();