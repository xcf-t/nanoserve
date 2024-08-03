const fileList = document.getElementById("file-list");
const dirTemplate = document.getElementById("dir-template");
const fileTemplate = document.getElementById("file-template");
let path = ".";
const pre = window.location.hash.substring(1);
if (pre.startsWith(".")) {
    path = pre;
}

const fileTypes = ["aac", "ai", "bmp", "cs", "css", "csv", "doc", "docx", "exe", "gif", "heic", "html", "java", "jpg", "js", "json", "jsx", "key", "m4p", "md", "mdx", "mov", "mp3", "mp4", "otf", "pdf", "php", "png", "ppt", "pptx", "psd", "py", "raw", "rb", "sass", "scss", "sh", "sql", "svg", "tiff", "tsx", "ttf", "txt", "wav", "woff", "xls", "xlsx", "xml", "yml"];

function createFile(file) {
    const template = fileTemplate.content.cloneNode(true);
    const link = template.querySelector("a");
    link.href = `/files/${path}/${(file.name)}`;
    link.download = "";
    template.querySelector(".file-name").innerText = file.name;
    template.querySelector(".file-size").innerText = formatBytes(file.size);
    let type = "file-earmark";
    for (const fileType of fileTypes) {
        if (file.name.endsWith(fileType)) {
            type = "filetype-" + fileType;
            break;
        }
    }
    template.querySelector("i").classList.add(`bi-${type}`);
    fileList.appendChild(template);
}

function createDir(dir) {
    const template = dirTemplate.content.cloneNode(true);
    const link = template.querySelector("a");
    link.onclick = function() {
        path += "/" + encodeURIComponent(dir.name);
        fileList.innerHTML = "";
        update();
    };
    link.href = `#${path}/${encodeURIComponent(dir.name)}`
    template.querySelector(".file-name").innerText = dir.name;
    const cls = dir.sym ? "folder-symlink" : "folder";
    template.querySelector("i").classList.add(`bi-${cls}`);
    fileList.appendChild(template);
}

function createBack() {
    const template = dirTemplate.content.cloneNode(true);
    template.querySelector("i").classList.add(`bi-folder-symlink`);
    template.querySelector(".file-name").innerText = "..";
    const link = template.querySelector("a");
    const idx = path.lastIndexOf("/");
    let newPath;
    if (idx === -1)
        newPath = ".";
    else
        newPath = path.substring(0, idx);
    link.onclick = function(e) {
        path = newPath;
        fileList.innerHTML = "";
        update();
    };
    link.href = `#${newPath}`;
    fileList.appendChild(template);
}

async function update() {
    const query = new URLSearchParams();
    query.set("path", path);
    const res = await fetch("/files?" + query, {
        method: "POST"
    }).then(res => res.json());


    res.sort((a,b) => {
        return b.dir - a.dir || a.name.localeCompare(b.name)
    });

    if (path.indexOf("/") !== -1) {
        createBack();
    }

    for (const file of res) {
        if (file.dir) {
            createDir(file);
        } else {
            createFile(file);
        }
    }
}
window.addEventListener("load", update)