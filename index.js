import init, { Cartridge, Cpu } from './chip8.js'

const CANVAS_WIDTH = 64;
const CANVAS_HEIGHT = 32;

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

const mainCtx = initCanvas(CANVAS_WIDTH, CANVAS_HEIGHT);
(async function run() {
    await init();

    const emulator = Cpu.new();

    const response = await window.fetch(`roms/PONG`);
    const program = await response.arrayBuffer();
    const cartridge = Cartridge.new(new Uint8Array(program));
    emulator.load_cartridge(cartridge);

    let running = false;
    const runloop = () => {
        if (running) {
            let result;
            // batch instructions
            for (let i = 0; i < 10; i++) {
                result = emulator.execute_cycle();
            }
            const displayState = result.get_display_state();
            updateCanvas(displayState, mainCtx, CANVAS_WIDTH, CANVAS_HEIGHT);
        }
        window.requestAnimationFrame(runloop);
    }

    window.requestAnimationFrame(runloop);

    const runButton = document.getElementById("run");
    runButton.addEventListener("click", () => {
        if (running) {
            running = false;
            runButton.innerHTML = "Start";
        } else {
            running = true;
            runButton.innerHTML = "Stop";
        }
    });
})();