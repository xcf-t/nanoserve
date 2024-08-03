const progressContainer = document.getElementById("progress-container");
const progressTemplate = document.getElementById("progress-template");
const dropContainer = document.getElementById("drop-container");
const uploadButton = document.getElementById("upload-button");
const fileInput = document.getElementById("drop-zone");
dropContainer.addEventListener("dragover", e => e.preventDefault());
dropContainer.addEventListener("dragenter", () => dropContainer.classList.add("drag-active"));
dropContainer.addEventListener("dragleave", () => dropContainer.classList.remove("drag-active"));
dropContainer.addEventListener("drop", e => {
    e.preventDefault();
    dropContainer.classList.remove("drag-active")

    fileInput.files = e.dataTransfer.files;
});

const chunkSize = 8 * 1024 * 1024;
async function createListener(file) {
    const uid = crypto.randomUUID();
    const progress = progressTemplate.content.cloneNode(true);
    progress.querySelector(".progress-filename").innerText = `${file.name} (${formatBytes(file.size)})`;
    progress.querySelector(".progress-box").id = uid;
    progressContainer.appendChild(progress);
    const bar = document.getElementById(uid);
    const inner = bar.querySelector(".progress-bar");

    const id = await fetch("/file", {
        method: "POST",
        body: JSON.stringify({
            size: file.size,
            chunk: chunkSize,
            name: file.name
        })
    }).then(res => res.text());

    function calculateProgress(offset, size) {
        let val = Math.min(offset, size) / size * 10000;
        return Math.round(val) / 100 + "%";
    }

    let blob, offset = 0;
    let done = 0;
    let error = false;
    while (offset < file.size && !error) {
        blob = file.slice(offset, offset + chunkSize);
        offset += blob.size;

        const task = fetch("/file/" + id, {
            method: "PUT",
            body: blob,
            headers: {
                //Range: `bytes=${offset - blob.size}-${offset}`
            }
        }).then(res => {
            if (res.status !== 204) {
                error = true;
            }

            done += blob.size;

            inner.style.width = calculateProgress(done, file.size);
            inner.innerText = calculateProgress(done, file.size);
        });

        await task;
    }

    const res = await fetch("/file/" + id, {
        method: "POST",
    });

    inner.style.width = "100%";
    inner.classList.remove("progress-bar-animated");
    if (res.status === 200 && !error) {
        inner.innerText = "100%";
        inner.classList.add("bg-success");
    } else {
        inner.innerText = "Upload Failed";
        inner.classList.add("bg-danger");
    }
}

uploadButton.addEventListener("click", async () => {
    console.log("Click")
    const tasks = [];
    for (const file of fileInput.files) {
        const task = createListener(file);
        tasks.push(task);
    }
    await Promise.all(tasks);
});