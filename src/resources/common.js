function formatBytes(value, unit = "B", prefix = ["", "ki", "Mi", "Gi", "Ti", "Ei"]) {
    if (value === 0) {
        return `${value} ${unit}`;
    }
    let mag = Math.floor(Math.log2(value));
    while (mag % 10 !== 0 && mag > 0) mag--;
    return `${Math.round(10 * value / Math.pow(1024, mag / 10))/10} ${prefix[mag / 10]}${unit}`;
}

const ttl = parseInt(sessionStorage.getItem("ttl"));
const state = sessionStorage.getItem("state");
if (state && ttl && (Date.now() - ttl) < 1000 * 60) {
    const values = JSON.parse(state);
    document.body.dataset.upload = values.upload;
    document.body.dataset.download = values.download;
} else {
    fetch("/status").then(res => res.json()).then(res => {
        document.body.dataset.upload = res.upload;
        document.body.dataset.download = res.download;
        sessionStorage.setItem("state", JSON.stringify(res));
        sessionStorage.setItem("ttl", String(Date.now()));
    });
}
