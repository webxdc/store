function d(btn, appId) {
    btn.innerText = "Downloading...";
    btn.style.background = "#bfbfbf";
    btn.setAttribute("onclick", "");
    let cmd = "/download " + appId;
    window.webxdc.sendUpdate({ payload: { simplebot: { text: cmd } } }, cmd);
}

function h(tag, attributes, ...children) {
    const element = document.createElement(tag);
    if (attributes) {
        Object.entries(attributes).forEach(entry => {
            element.setAttribute(entry[0], entry[1]);
        });
    }
    element.append(...children);
    return element;
}

function main(data) {
    let root = document.getElementById("root");
    data.forEach(meta => {
        root.appendChild(h("div", { class: "card" },
            h("div", { class: "flex" },
                h("img", { src: meta.icon }),
                h("div", { class: "flex-1" },
                    h("h3", {}, meta.name),
                    h("small", {},
                        h("em", {}, "by ", meta.publisher, h("br"), meta.version)
                    )
                )
            ),
            h("p", {}, meta.description),
            h("br"),
            h("div", { class: "right" }, h("a", { class: "btn", href: "#", onclick: "d(this, '" + meta.id + "'); return false;" }, "Download"))
        ));
    });
}

onload = () => {
    fetch("data.json")
        .then(response => response.json())
        .then(json => main(json));
};