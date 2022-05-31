import * as wasm from "magic-wormhole-wasm";

const fileInput = document.getElementById("file-input")
const codeInput = document.getElementById("code-input")
const startButton = document.getElementById("button-start")

const wormholeConfig = wasm.WormholeConfig.new(
    "wss://relay.magic-wormhole.io:443/v1",
    "wss://piegames.de/wormhole-transit-relay"
)

function downloadFile(data, fileName) {
    const url = window.URL.createObjectURL(new Blob([new Uint8Array(data)]));
    const a = document.createElement('a');
    a.style.display = 'none';
    a.href = url;
    a.download = fileName;
    document.body.appendChild(a);
    a.click();
    window.URL.revokeObjectURL(url);
}

function cancelPromise() {
    let cancel;
    let promise = new Promise(resolve => {
        cancel = resolve
    });

    return [promise, cancel];
}

startButton.addEventListener('click', () => {
    const code = codeInput.value;

    if (!code) {
        alert("Please enter a code")
    } else {
        const [promise, cancel] = cancelPromise();

        wasm.receive(wormholeConfig, code, promise, event => {
            console.log(event)
        })
            .then(x => {
                console.log("receiving finished", x);
                if (x) {
                    const {data, filename, filesize} = x;
                    downloadFile(data, filename)
                }
            })
            .catch(err => {
                console.error(err)
            })
    }
})

fileInput.addEventListener('input', () => {
    const [promise, cancel] = cancelPromise();

    const fileList = fileInput.files;
    if (fileList.length < 1 || !fileList[0]) {
        console.error("Please select a valid file")
        return
    }

    const file = fileList[0]

    wasm.send(wormholeConfig, file, file.name, promise, event => console.log(event))
        .then(x => {
            console.log("sending finished");
        })
        .catch(err => {
            console.error(err)
        })
})


